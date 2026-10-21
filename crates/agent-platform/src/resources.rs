use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use sysinfo::{Disks, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourceSnapshot {
    pub cpu_usage_percent: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub memory_free_mb: u64,
    pub memory_used_percent: f32,
    pub disk_free_gb: f64,
    pub process_count: usize,
    pub timestamp: DateTime<Utc>,
}

pub struct SystemResourceMonitor {
    sys: Mutex<System>,
    disks: Mutex<Disks>,
}

impl SystemResourceMonitor {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        sys.refresh_memory();
        let disks = Disks::new_with_refreshed_list();

        Self {
            sys: Mutex::new(sys),
            disks: Mutex::new(disks),
        }
    }

    pub fn get_snapshot(&self) -> SystemResourceSnapshot {
        let mut sys = self.sys.lock().unwrap();
        sys.refresh_cpu_all();
        sys.refresh_memory();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let cpus = sys.cpus();
        let cpu_usage_percent: f32 = if !cpus.is_empty() {
            cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / (cpus.len() as f32)
        } else {
            0.0
        };

        let memory_total_mb = sys.total_memory() / (1024 * 1024);
        let memory_used_mb = sys.used_memory() / (1024 * 1024);
        let memory_free_mb = sys.free_memory() / (1024 * 1024);
        let memory_used_percent = if memory_total_mb > 0 {
            (memory_used_mb as f32 / memory_total_mb as f32) * 100.0
        } else {
            0.0
        };

        let mut disks = self.disks.lock().unwrap();
        disks.refresh(true);
        let disk_free_bytes: u64 = disks.iter().map(|d| d.available_space()).sum();
        let disk_free_gb = disk_free_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

        let process_count = sys.processes().len();

        SystemResourceSnapshot {
            cpu_usage_percent,
            memory_used_mb,
            memory_total_mb,
            memory_free_mb,
            memory_used_percent,
            disk_free_gb,
            process_count,
            timestamp: Utc::now(),
        }
    }

    /// Determines whether the system is under critical memory or CPU pressure
    pub fn is_under_backpressure(&self, max_memory_percent: f32, max_cpu_percent: f32) -> bool {
        let snapshot = self.get_snapshot();
        snapshot.memory_used_percent >= max_memory_percent || snapshot.cpu_usage_percent >= max_cpu_percent
    }
}

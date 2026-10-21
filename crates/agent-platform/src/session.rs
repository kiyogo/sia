use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionInfo {
    pub session_id: u32,
    pub is_session_zero: bool,
    pub is_active: bool,
    pub is_locked: bool,
}

pub trait InteractiveSessionProvider: Send + Sync {
    fn get_current_session(&self) -> SessionInfo;
    fn is_interactive_desktop_available(&self) -> bool {
        let s = self.get_current_session();
        !s.is_session_zero && s.is_active && !s.is_locked
    }
}

pub struct WindowsSessionProvider;

impl WindowsSessionProvider {
    pub fn new() -> Self {
        Self
    }
}

impl InteractiveSessionProvider for WindowsSessionProvider {
    fn get_current_session(&self) -> SessionInfo {
        #[cfg(windows)]
        {
            // Query current process session ID via ProcessIdToSessionId
            let pid = std::process::id();
            let mut session_id: u32 = 0;
            unsafe {
                #[link(name = "kernel32")]
                extern "system" {
                    fn ProcessIdToSessionId(process_id: u32, session_id: *mut u32) -> i32;
                }
                let res = ProcessIdToSessionId(pid, &mut session_id);
                if res == 0 {
                    session_id = 1; // Fallback
                }
            }

            // Check if workstation is locked via logonui check
            let is_locked = {
                let mut cmd = std::process::Command::new("powershell.exe");
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    cmd.creation_flags(0x08000000);
                }
                let output = cmd
                    .args([
                        "-NoProfile",
                        "-NonInteractive",
                        "-Command",
                        "if (Get-Process -Name logonui -ErrorAction SilentlyContinue) { 'LOCKED' } else { 'UNLOCKED' }",
                    ])
                    .output();
                if let Ok(out) = output {
                    String::from_utf8_lossy(&out.stdout).contains("LOCKED")
                } else {
                    false
                }
            };

            SessionInfo {
                session_id,
                is_session_zero: session_id == 0,
                is_active: session_id > 0,
                is_locked,
            }
        }
        #[cfg(not(windows))]
        {
            SessionInfo {
                session_id: 1,
                is_session_zero: false,
                is_active: true,
                is_locked: false,
            }
        }
    }
}

pub struct MockSessionProvider {
    pub session: std::sync::RwLock<SessionInfo>,
}

impl MockSessionProvider {
    pub fn new(session_id: u32, is_active: bool, is_locked: bool) -> Self {
        Self {
            session: std::sync::RwLock::new(SessionInfo {
                session_id,
                is_session_zero: session_id == 0,
                is_active,
                is_locked,
            }),
        }
    }

    pub fn set_locked(&self, locked: bool) {
        let mut s = self.session.write().unwrap();
        s.is_locked = locked;
    }

    pub fn set_session_zero(&self, session_zero: bool) {
        let mut s = self.session.write().unwrap();
        s.is_session_zero = session_zero;
        s.session_id = if session_zero { 0 } else { 1 };
    }
}

impl InteractiveSessionProvider for MockSessionProvider {
    fn get_current_session(&self) -> SessionInfo {
        self.session.read().unwrap().clone()
    }
}

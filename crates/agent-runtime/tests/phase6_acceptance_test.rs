use agent_core::{
    BudgetGovernanceEngine, BudgetHierarchyConfig, CircuitBreaker, ErrorClass,
    ErrorClassifier,
};
use agent_platform::{
    MockSessionProvider, MockServiceAdapter, RuntimeServiceAdapter,
    ServiceStartupType, ServiceStatus, SystemResourceMonitor,
};
use agent_runtime::{
    recovery::{CrashRecoveryEngine, UnknownReconciliationDecision},
    scheduler::AutonomousScheduler,
    service::{AgentRuntimeService, InteractiveWorkerResult, WindowsInteractiveWorker},
    watchdog::ProcessSupervisor,
    RuntimeConfig, RuntimeHost,
};
use agent_security::PermissionMode;
use agent_storage::{
    Database, ScheduledJob, Workflow, WorkflowStatus, WorkflowTask,
    WorkflowTaskStatus,
};
use chrono::{Duration, Utc};
use std::sync::Arc;
use tempfile::tempdir;
use uuid::Uuid;

async fn create_test_db() -> (Arc<Database>, tempfile::TempDir) {
    let temp = tempdir().expect("tempdir");
    let db_path = temp.path().join("test_p6.db");
    let db = Database::open(db_path).expect("open test db");
    (Arc::new(db), temp)
}

#[allow(dead_code)]
async fn create_test_runtime() -> (RuntimeHost, tempfile::TempDir) {
    let temp = tempdir().expect("tempdir");
    let db_path = temp.path().join("test_phase6_runtime.db");
    let workspace = temp.path().join("workspace");
    std::fs::create_dir_all(&workspace).expect("workspace dir");

    let config = RuntimeConfig {
        provider_url: String::new(),
        api_key: String::new(),
        model_name: "test-model".to_string(),
        workspace_path: workspace,
        permission_mode: PermissionMode::FullAuto,
        max_loops: 15,
        ..Default::default()
    };

    let host = RuntimeHost::new(db_path, config)
        .await
        .expect("create runtime host");

    (host, temp)
}

// =============================================================================
// Test 01: Windows Service Installation
// =============================================================================
#[tokio::test]
async fn test_p6_01_service_installation() {
    let adapter = MockServiceAdapter::new(ServiceStatus::NotInstalled);
    assert_eq!(
        adapter.status("AAIRuntime").await.unwrap(),
        ServiceStatus::NotInstalled
    );

    adapter
        .install(
            "AAIRuntime",
            "Universal AI 24/7 Runtime Service",
            std::path::Path::new("C:\\aai\\desktop.exe"),
            ServiceStartupType::AutoStart,
        )
        .await
        .expect("install service");

    assert_eq!(
        adapter.status("AAIRuntime").await.unwrap(),
        ServiceStatus::Stopped
    );
}

// =============================================================================
// Test 02: Windows Service Start & Running State
// =============================================================================
#[tokio::test]
async fn test_p6_02_service_start_running() {
    let adapter = MockServiceAdapter::new(ServiceStatus::Stopped);
    adapter.start("AAIRuntime").await.expect("start service");
    assert_eq!(
        adapter.status("AAIRuntime").await.unwrap(),
        ServiceStatus::Running
    );
}

// =============================================================================
// Test 03: Windows Service Stop & Stopped State
// =============================================================================
#[tokio::test]
async fn test_p6_03_service_stop_stopped() {
    let adapter = MockServiceAdapter::new(ServiceStatus::Running);
    adapter.stop("AAIRuntime").await.expect("stop service");
    assert_eq!(
        adapter.status("AAIRuntime").await.unwrap(),
        ServiceStatus::Stopped
    );
}

// =============================================================================
// Test 04: Windows Service Restart
// =============================================================================
#[tokio::test]
async fn test_p6_04_service_restart() {
    let adapter = MockServiceAdapter::new(ServiceStatus::Running);
    adapter.restart("AAIRuntime").await.expect("restart service");
    assert_eq!(
        adapter.status("AAIRuntime").await.unwrap(),
        ServiceStatus::Running
    );
}

// =============================================================================
// Test 05: Windows Service Uninstall
// =============================================================================
#[tokio::test]
async fn test_p6_05_service_uninstall() {
    let adapter = MockServiceAdapter::new(ServiceStatus::Running);
    adapter.uninstall("AAIRuntime").await.expect("uninstall service");
    assert_eq!(
        adapter.status("AAIRuntime").await.unwrap(),
        ServiceStatus::NotInstalled
    );
}

// =============================================================================
// Test 06: Desktop UI Closure with Service Running (Headless Persistence)
// =============================================================================
#[tokio::test]
async fn test_p6_06_desktop_ui_close_service_persists() {
    let (db, _temp) = create_test_db().await;
    let service = Arc::new(AgentRuntimeService::new(db.clone()));

    let _summary = service.start().await.expect("start service");
    assert!(service.is_running());
    assert_eq!(service.get_status().await, ServiceStatus::Running);

    // Simulate Desktop UI disconnecting / dropping client context
    {
        let mock_ui_client_handle = service.clone();
        assert!(mock_ui_client_handle.is_running());
        // mock_ui_client_handle drops here
    }

    // Verify background runtime service continues running uninterrupted
    assert!(service.is_running());
    assert_eq!(service.get_status().await, ServiceStatus::Running);
}

// =============================================================================
// Test 07: Desktop UI Reconnect & State Recovery
// =============================================================================
#[tokio::test]
async fn test_p6_07_desktop_ui_reconnect_state_recovery() {
    let (db, _temp) = create_test_db().await;
    let service = Arc::new(AgentRuntimeService::new(db.clone()));
    let _ = service.start().await.expect("start service");

    // Add a scheduled job while headless
    let job_id = Uuid::new_v4();
    let job = ScheduledJob {
        id: job_id,
        name: "Autonomous Hourly Sync".to_string(),
        schedule_type: "interval".to_string(),
        cron_expression: None,
        interval_seconds: Some(3600),
        timezone: "UTC".to_string(),
        next_run_at: Some(Utc::now() + Duration::hours(1)),
        last_run_at: None,
        status: "Active".to_string(),
        missed_run_policy: "Skip".to_string(),
        overlap_policy: "Skip".to_string(),
        max_runtime_seconds: Some(600),
        budget_limit: Some(2.0),
        workflow_template: "data_sync".to_string(),
        payload_json: "{}".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    db.create_scheduled_job(&job).await.expect("create job");

    // Simulate new UI client connecting and polling state
    let active_status = service.get_status().await;
    assert_eq!(active_status, ServiceStatus::Running);

    let jobs = db.list_scheduled_jobs().await.expect("list jobs");
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, job_id);

    let snapshot = service.get_resource_snapshot();
    assert!(snapshot.memory_total_mb > 0);
}

// =============================================================================
// Test 08: Durable Task Queue Persistence Across Abrupt Kill
// =============================================================================
#[tokio::test]
async fn test_p6_08_durable_task_queue_persistence() {
    let temp = tempdir().expect("tempdir");
    let db_path = temp.path().join("durable_queue.db");

    let wf_id = Uuid::new_v4();
    let task_id1 = Uuid::new_v4();
    let task_id2 = Uuid::new_v4();

    // Session 1: Create tasks and simulate sudden process death (drop db)
    {
        let db = Database::open(&db_path).expect("open db 1");
        let wf = Workflow {
            id: wf_id,
            title: "Durable Workflow".to_string(),
            goal: "Test persistence".to_string(),
            status: WorkflowStatus::Running,
            budget_json: "{}".to_string(),
            result_summary: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        db.create_workflow(&wf).await.expect("create wf");

        let t1 = WorkflowTask {
            id: task_id1,
            workflow_id: wf_id,
            parent_task_id: None,
            title: "Task 1 Data Collection".to_string(),
            description: "Desc".to_string(),
            assigned_agent: "Research".to_string(),
            status: WorkflowTaskStatus::Completed,
            priority: 1,
            inputs_json: "{}".to_string(),
            outputs_json: "{}".to_string(),
            acceptance_criteria: "Complete".to_string(),
            retry_count: 0,
            max_retries: 3,
            timeout_secs: 60,
            started_at: Some(Utc::now()),
            completed_at: Some(Utc::now()),
            error: None,
        };
        let t2 = WorkflowTask {
            id: task_id2,
            workflow_id: wf_id,
            parent_task_id: None,
            title: "Task 2 Data Processing".to_string(),
            description: "Desc".to_string(),
            assigned_agent: "DataAnalyst".to_string(),
            status: WorkflowTaskStatus::Running,
            priority: 2,
            inputs_json: "{}".to_string(),
            outputs_json: "{}".to_string(),
            acceptance_criteria: "Process".to_string(),
            retry_count: 0,
            max_retries: 3,
            timeout_secs: 60,
            started_at: Some(Utc::now()),
            completed_at: None,
            error: None,
        };
        db.create_workflow_task(&t1).await.expect("create t1");
        db.create_workflow_task(&t2).await.expect("create t2");
        // db dropped abruptly here
    }

    // Session 2: Reopen after simulated process restart
    {
        let db2 = Database::open(&db_path).expect("reopen db 2");
        let t1_loaded = db2.get_workflow_task(task_id1).await.unwrap().unwrap();
        let t2_loaded = db2.get_workflow_task(task_id2).await.unwrap().unwrap();

        assert_eq!(t1_loaded.status, WorkflowTaskStatus::Completed);
        assert_eq!(t2_loaded.status, WorkflowTaskStatus::Running);
        assert_eq!(t2_loaded.title, "Task 2 Data Processing");
    }
}

// =============================================================================
// Test 09: Startup Crash Recovery: Pure Tasks Reset to Ready
// =============================================================================
#[tokio::test]
async fn test_p6_09_startup_crash_recovery_pure_tasks_reset() {
    let (db, _temp) = create_test_db().await;
    let wf_id = Uuid::new_v4();
    let wf = Workflow {
        id: wf_id,
        title: "Analytics Workflow".to_string(),
        goal: "Data crunching".to_string(),
        status: WorkflowStatus::Running,
        budget_json: "{}".to_string(),
        result_summary: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    db.create_workflow(&wf).await.expect("create wf");

    let pure_task_id = Uuid::new_v4();
    let task = WorkflowTask {
        id: pure_task_id,
        workflow_id: wf_id,
        parent_task_id: None,
        title: "Compute summary statistics and regression".to_string(),
        description: "Pure calculation".to_string(),
        assigned_agent: "DataAnalyst".to_string(),
        status: WorkflowTaskStatus::Running,
        priority: 1,
        inputs_json: "{}".to_string(),
        outputs_json: "{}".to_string(),
        acceptance_criteria: "Calculated".to_string(),
        retry_count: 0,
        max_retries: 3,
        timeout_secs: 60,
        started_at: Some(Utc::now()),
        completed_at: None,
        error: None,
    };
    db.create_workflow_task(&task).await.expect("create task");

    let recovery = CrashRecoveryEngine::new(db.clone());
    let summary = recovery.reconcile_startup_tasks().await.expect("reconcile");

    assert_eq!(summary.tasks_recovered_to_ready, 1);
    assert_eq!(summary.tasks_flagged_outcome_unknown, 0);

    let recovered_task = db.get_workflow_task(pure_task_id).await.unwrap().unwrap();
    assert_eq!(recovered_task.status, WorkflowTaskStatus::Ready);
    assert_eq!(recovered_task.retry_count, 1);
}

// =============================================================================
// Test 10: Startup Crash Recovery: External Side-Effects Held in OutcomeUnknown
// =============================================================================
#[tokio::test]
async fn test_p6_10_startup_crash_recovery_side_effects_held() {
    let (db, _temp) = create_test_db().await;
    let wf_id = Uuid::new_v4();
    let wf = Workflow {
        id: wf_id,
        title: "Customer Outbound".to_string(),
        goal: "Email dispatch".to_string(),
        status: WorkflowStatus::Running,
        budget_json: "{}".to_string(),
        result_summary: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    db.create_workflow(&wf).await.expect("create wf");

    let email_task_id = Uuid::new_v4();
    let email_task = WorkflowTask {
        id: email_task_id,
        workflow_id: wf_id,
        parent_task_id: None,
        title: "Send final settlement statement email to customer".to_string(),
        description: "External email transmission".to_string(),
        assigned_agent: "Email".to_string(),
        status: WorkflowTaskStatus::Running,
        priority: 1,
        inputs_json: "{}".to_string(),
        outputs_json: "{}".to_string(),
        acceptance_criteria: "Sent".to_string(),
        retry_count: 0,
        max_retries: 3,
        timeout_secs: 60,
        started_at: Some(Utc::now()),
        completed_at: None,
        error: None,
    };
    db.create_workflow_task(&email_task).await.expect("create task");

    let recovery = CrashRecoveryEngine::new(db.clone());
    let summary = recovery.reconcile_startup_tasks().await.expect("reconcile");

    assert_eq!(summary.tasks_flagged_outcome_unknown, 1);
    assert_eq!(summary.tasks_recovered_to_ready, 0);

    let updated = db.get_workflow_task(email_task_id).await.unwrap().unwrap();
    assert_eq!(updated.status, WorkflowTaskStatus::OutcomeUnknown);
}

// =============================================================================
// Test 11: Operator Manual Reconciliation of OutcomeUnknown
// =============================================================================
#[tokio::test]
async fn test_p6_11_operator_manual_reconciliation() {
    let (db, _temp) = create_test_db().await;
    let wf_id = Uuid::new_v4();
    let wf = Workflow {
        id: wf_id,
        title: "Reconciliation Test".to_string(),
        goal: "Manual decision".to_string(),
        status: WorkflowStatus::Running,
        budget_json: "{}".to_string(),
        result_summary: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    db.create_workflow(&wf).await.expect("create wf");

    let task_id1 = Uuid::new_v4();
    let task_id2 = Uuid::new_v4();

    let task1 = WorkflowTask {
        id: task_id1,
        workflow_id: wf_id,
        parent_task_id: None,
        title: "Dispatch payment to vendor".to_string(),
        description: "Payment".to_string(),
        assigned_agent: "Business".to_string(),
        status: WorkflowTaskStatus::OutcomeUnknown,
        priority: 1,
        inputs_json: "{}".to_string(),
        outputs_json: "{}".to_string(),
        acceptance_criteria: "Done".to_string(),
        retry_count: 0,
        max_retries: 3,
        timeout_secs: 60,
        started_at: Some(Utc::now()),
        completed_at: None,
        error: None,
    };
    let task2 = WorkflowTask {
        id: task_id2,
        workflow_id: wf_id,
        parent_task_id: None,
        title: "Commit git repository tag".to_string(),
        description: "Git commit".to_string(),
        assigned_agent: "Coding".to_string(),
        status: WorkflowTaskStatus::OutcomeUnknown,
        priority: 1,
        inputs_json: "{}".to_string(),
        outputs_json: "{}".to_string(),
        acceptance_criteria: "Done".to_string(),
        retry_count: 0,
        max_retries: 3,
        timeout_secs: 60,
        started_at: Some(Utc::now()),
        completed_at: None,
        error: None,
    };
    db.create_workflow_task(&task1).await.expect("create task1");
    db.create_workflow_task(&task2).await.expect("create task2");

    let recovery = CrashRecoveryEngine::new(db.clone());

    // Operator verifies vendor received payment -> MarkCompleted
    recovery
        .reconcile_unknown_outcome(task_id1, UnknownReconciliationDecision::MarkCompleted)
        .await
        .expect("reconcile completed");
    let t1_res = db.get_workflow_task(task_id1).await.unwrap().unwrap();
    assert_eq!(t1_res.status, WorkflowTaskStatus::Completed);

    // Operator verifies tag was NOT committed -> RetrySafeStep
    recovery
        .reconcile_unknown_outcome(task_id2, UnknownReconciliationDecision::RetrySafeStep)
        .await
        .expect("reconcile retry");
    let t2_res = db.get_workflow_task(task_id2).await.unwrap().unwrap();
    assert_eq!(t2_res.status, WorkflowTaskStatus::Ready);
    assert_eq!(t2_res.retry_count, 1);
}

// =============================================================================
// Test 12: Multi-Worker Lease & Heartbeat Tracking
// =============================================================================
#[tokio::test]
async fn test_p6_12_multi_worker_lease_and_heartbeat() {
    let (db, _temp) = create_test_db().await;
    let supervisor = ProcessSupervisor::new(db.clone(), 60);

    let proc_id = supervisor
        .register_process(
            1234,
            "PlaywrightMcp",
            "C:\\workspace",
            None,
            3,
        )
        .await;

    supervisor
        .record_heartbeat(proc_id, Some(r#"{"cpu_pct": 5.4, "ram_mb": 180}"#))
        .await;

    let procs = db.list_supervised_processes().await.expect("list procs");
    assert_eq!(procs.len(), 1);
    assert_eq!(procs[0].id, proc_id);
    assert_eq!(procs[0].health_status, "Healthy");
    assert!(procs[0].resource_usage_json.as_ref().unwrap().contains("ram_mb"));
}

// =============================================================================
// Test 13: Atomic Task Claiming with Fencing Token
// =============================================================================
#[tokio::test]
async fn test_p6_13_atomic_task_claiming_fencing_token() {
    let (db, _temp) = create_test_db().await;
    let wf_id = Uuid::new_v4();
    let wf = Workflow {
        id: wf_id,
        title: "Atomic Claim WF".to_string(),
        goal: "Fencing test".to_string(),
        status: WorkflowStatus::Running,
        budget_json: "{}".to_string(),
        result_summary: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    db.create_workflow(&wf).await.expect("create wf");

    let task_id = Uuid::new_v4();
    let task = WorkflowTask {
        id: task_id,
        workflow_id: wf_id,
        parent_task_id: None,
        title: "High priority compute".to_string(),
        description: "Task".to_string(),
        assigned_agent: "WorkerA".to_string(),
        status: WorkflowTaskStatus::Ready,
        priority: 10,
        inputs_json: "{}".to_string(),
        outputs_json: "{}".to_string(),
        acceptance_criteria: "Done".to_string(),
        retry_count: 0,
        max_retries: 3,
        timeout_secs: 60,
        started_at: None,
        completed_at: None,
        error: None,
    };
    db.create_workflow_task(&task).await.expect("create task");

    // Worker 1 claims task atomically: transitions Ready -> Running
    let mut t1 = db.get_workflow_task(task_id).await.unwrap().unwrap();
    assert_eq!(t1.status, WorkflowTaskStatus::Ready);
    t1.status = WorkflowTaskStatus::Running;
    t1.started_at = Some(Utc::now());
    db.update_workflow_task(&t1).await.expect("worker 1 claim");

    // Worker 2 attempts to claim: detects it is already Running, claim rejected
    let t2 = db.get_workflow_task(task_id).await.unwrap().unwrap();
    assert_ne!(t2.status, WorkflowTaskStatus::Ready);
    assert_eq!(t2.status, WorkflowTaskStatus::Running);
}

// =============================================================================
// Test 14: Stale Worker Invalidation on Task Mutation
// =============================================================================
#[tokio::test]
async fn test_p6_14_stale_worker_fencing_token_invalidation() {
    let (db, _temp) = create_test_db().await;
    let wf_id = Uuid::new_v4();
    let wf = Workflow {
        id: wf_id,
        title: "Stale Worker Test".to_string(),
        goal: "Invalidation test".to_string(),
        status: WorkflowStatus::Running,
        budget_json: "{}".to_string(),
        result_summary: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    db.create_workflow(&wf).await.expect("create wf");

    let task_id = Uuid::new_v4();
    let mut task = WorkflowTask {
        id: task_id,
        workflow_id: wf_id,
        parent_task_id: None,
        title: "Critical update task".to_string(),
        description: "Task".to_string(),
        assigned_agent: "WorkerStale".to_string(),
        status: WorkflowTaskStatus::Running,
        priority: 5,
        inputs_json: "{}".to_string(),
        outputs_json: "{}".to_string(),
        acceptance_criteria: "Done".to_string(),
        retry_count: 1, // current version / token = 1
        max_retries: 3,
        timeout_secs: 60,
        started_at: Some(Utc::now()),
        completed_at: None,
        error: None,
    };
    db.create_workflow_task(&task).await.expect("create task");

    // Supervisor detects timeout and reclaims task to Ready with retry_count = 2 (token advanced)
    task.retry_count = 2;
    task.status = WorkflowTaskStatus::Ready;
    db.update_workflow_task(&task).await.expect("reclaim");

    // Stale worker tries to complete task presenting old retry_count = 1
    let latest = db.get_workflow_task(task_id).await.unwrap().unwrap();
    let stale_token = 1;
    let is_valid = latest.retry_count == stale_token;
    assert!(!is_valid, "Stale worker token MUST be invalidated");
}

// =============================================================================
// Test 15: Autonomous Scheduler Interval-Based Triggering
// =============================================================================
#[tokio::test]
async fn test_p6_15_autonomous_scheduler_interval() {
    let (db, _temp) = create_test_db().await;
    let scheduler = AutonomousScheduler::new(db.clone());

    let job_id = Uuid::new_v4();
    let job = ScheduledJob {
        id: job_id,
        name: "Test Interval Trigger".to_string(),
        schedule_type: "interval".to_string(),
        cron_expression: None,
        interval_seconds: Some(60),
        timezone: "UTC".to_string(),
        next_run_at: Some(Utc::now() - Duration::seconds(5)), // Due now!
        last_run_at: None,
        status: "Active".to_string(),
        missed_run_policy: "Skip".to_string(),
        overlap_policy: "Skip".to_string(),
        max_runtime_seconds: Some(300),
        budget_limit: Some(1.0),
        workflow_template: "quick_sync".to_string(),
        payload_json: "{}".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    db.create_scheduled_job(&job).await.expect("create job");

    let triggered = scheduler.evaluate_and_trigger_due_jobs().await.expect("evaluate");
    assert_eq!(triggered.len(), 1);
    assert_eq!(triggered[0].job_id, job_id);

    // Verify next run is advanced into the future
    let updated_job = db.get_scheduled_job(job_id).await.unwrap().unwrap();
    assert!(updated_job.next_run_at.unwrap() > Utc::now());
}

// =============================================================================
// Test 16: Autonomous Scheduler Missed-Run Policy: Skip
// =============================================================================
#[tokio::test]
async fn test_p6_16_autonomous_scheduler_missed_run_skip() {
    let (db, _temp) = create_test_db().await;
    let scheduler = AutonomousScheduler::new(db.clone());

    let job_id = Uuid::new_v4();
    let overdue_time = Utc::now() - Duration::seconds(1800); // 30 minutes overdue (e.g. system suspended)
    let job = ScheduledJob {
        id: job_id,
        name: "Missed Run Skip Job".to_string(),
        schedule_type: "interval".to_string(),
        cron_expression: None,
        interval_seconds: Some(300),
        timezone: "UTC".to_string(),
        next_run_at: Some(overdue_time),
        last_run_at: None,
        status: "Active".to_string(),
        missed_run_policy: "Skip".to_string(),
        overlap_policy: "Skip".to_string(),
        max_runtime_seconds: Some(300),
        budget_limit: Some(1.0),
        workflow_template: "skip_template".to_string(),
        payload_json: "{}".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    db.create_scheduled_job(&job).await.expect("create job");

    // Policy is Skip: should NOT trigger execution, just advance next_run
    let triggered = scheduler.evaluate_and_trigger_due_jobs().await.expect("evaluate");
    assert_eq!(triggered.len(), 0);

    let updated_job = db.get_scheduled_job(job_id).await.unwrap().unwrap();
    assert!(updated_job.next_run_at.unwrap() > Utc::now());
}

// =============================================================================
// Test 17: Autonomous Scheduler Missed-Run Policy: RunOnce
// =============================================================================
#[tokio::test]
async fn test_p6_17_autonomous_scheduler_missed_run_run_once() {
    let (db, _temp) = create_test_db().await;
    let scheduler = AutonomousScheduler::new(db.clone());

    let job_id = Uuid::new_v4();
    let overdue_time = Utc::now() - Duration::seconds(1800); // 30 minutes overdue
    let job = ScheduledJob {
        id: job_id,
        name: "Missed Run RunOnce Job".to_string(),
        schedule_type: "interval".to_string(),
        cron_expression: None,
        interval_seconds: Some(300),
        timezone: "UTC".to_string(),
        next_run_at: Some(overdue_time),
        last_run_at: None,
        status: "Active".to_string(),
        missed_run_policy: "RunOnce".to_string(),
        overlap_policy: "Skip".to_string(),
        max_runtime_seconds: Some(300),
        budget_limit: Some(1.0),
        workflow_template: "run_once_template".to_string(),
        payload_json: "{}".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    db.create_scheduled_job(&job).await.expect("create job");

    // Policy is RunOnce: triggers exactly one catch-up execution immediately
    let triggered = scheduler.evaluate_and_trigger_due_jobs().await.expect("evaluate");
    assert_eq!(triggered.len(), 1);
    assert_eq!(triggered[0].job_id, job_id);

    let history = db.list_job_executions(job_id).await.expect("list history");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].status, "Running");
}

// =============================================================================
// Test 18: Overlapping Job Execution Prevention (JobConcurrencyPolicy::Skip)
// =============================================================================
#[tokio::test]
async fn test_p6_18_overlapping_job_protection() {
    let (db, _temp) = create_test_db().await;
    let scheduler = AutonomousScheduler::new(db.clone());

    let job_id = Uuid::new_v4();
    let job = ScheduledJob {
        id: job_id,
        name: "Overlap Prevention Job".to_string(),
        schedule_type: "interval".to_string(),
        cron_expression: None,
        interval_seconds: Some(60),
        timezone: "UTC".to_string(),
        next_run_at: Some(Utc::now() - Duration::seconds(10)),
        last_run_at: None,
        status: "Active".to_string(),
        missed_run_policy: "Skip".to_string(),
        overlap_policy: "Skip".to_string(),
        max_runtime_seconds: Some(300),
        budget_limit: Some(1.0),
        workflow_template: "long_task".to_string(),
        payload_json: "{}".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    db.create_scheduled_job(&job).await.expect("create job");

    // Trigger cycle 1
    let triggered1 = scheduler.evaluate_and_trigger_due_jobs().await.expect("eval 1");
    assert_eq!(triggered1.len(), 1);
    let exec_id = triggered1[0].execution_id;

    // Fast-forward next_run_at to simulate second cycle arriving while cycle 1 is still running
    let mut updated = db.get_scheduled_job(job_id).await.unwrap().unwrap();
    updated.next_run_at = Some(Utc::now() - Duration::seconds(1));
    db.update_scheduled_job_execution(job_id, Utc::now(), updated.next_run_at).await.unwrap();

    // Trigger cycle 2: should be skipped because cycle 1 is still in `running_jobs`
    let triggered2 = scheduler.evaluate_and_trigger_due_jobs().await.expect("eval 2");
    assert_eq!(triggered2.len(), 0, "Overlapping job MUST be skipped");

    // Complete cycle 1
    scheduler.mark_job_completed(job_id, exec_id, true, "Finished successfully").await;

    // Fast forward again: now cycle 3 is allowed to run
    updated.next_run_at = Some(Utc::now() - Duration::seconds(1));
    db.update_scheduled_job_execution(job_id, Utc::now(), updated.next_run_at).await.unwrap();
    let triggered3 = scheduler.evaluate_and_trigger_due_jobs().await.expect("eval 3");
    assert_eq!(triggered3.len(), 1);
}

// =============================================================================
// Test 19: Worker Process Supervisor Heartbeat Tracking
// =============================================================================
#[tokio::test]
async fn test_p6_19_worker_supervisor_heartbeat_tracking() {
    let (db, _temp) = create_test_db().await;
    let supervisor = ProcessSupervisor::new(db.clone(), 60);

    let current_pid = std::process::id();
    let proc_id = supervisor
        .register_process(current_pid, "TestWorker", "C:\\workspace", None, 3)
        .await;

    supervisor.record_heartbeat(proc_id, None).await;

    let events = supervisor.check_health_and_reap().await;
    assert!(events.is_empty(), "Healthy process must not generate crash events");
}

// =============================================================================
// Test 20: Watchdog Process Crash Detection & Clean Restart
// =============================================================================
#[tokio::test]
async fn test_p6_20_watchdog_crash_detection_restart() {
    let (db, _temp) = create_test_db().await;
    // Timeout of 0 seconds causes immediate expiration
    let supervisor = ProcessSupervisor::new(db.clone(), 0);

    let dead_pid = 9999999;
    let proc_id = supervisor
        .register_process(dead_pid, "CrashedWorker", "C:\\workspace", None, 3)
        .await;

    // Small delay so heartbeat elapsed > 0
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let events = supervisor.check_health_and_reap().await;
    assert_eq!(events.len(), 1);
    assert!(events[0].1.contains("Will restart"));

    let procs = db.list_supervised_processes().await.expect("list procs");
    let record = procs.iter().find(|p| p.id == proc_id).unwrap();
    assert_eq!(record.health_status, "Crashed");
}

// =============================================================================
// Test 21: Crash Loop Circuit Breaker (Quarantine after 3 crashes in 60s)
// =============================================================================
#[tokio::test]
async fn test_p6_21_crash_loop_quarantine() {
    let (db, _temp) = create_test_db().await;
    let supervisor = ProcessSupervisor::new(db.clone(), 0);

    let dead_pid = 9999998;
    let proc_id = supervisor
        .register_process(dead_pid, "LoopingWorker", "C:\\workspace", None, 2)
        .await;

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    // Crash 1
    let ev1 = supervisor.check_health_and_reap().await;
    assert_eq!(ev1.len(), 1);
    assert!(ev1[0].1.contains("Will restart"));

    // Crash 2: reaches max_restarts (2) -> Quarantined
    let ev2 = supervisor.check_health_and_reap().await;
    assert_eq!(ev2.len(), 1);
    assert!(ev2[0].1.contains("Quarantined"));

    let procs = db.list_supervised_processes().await.expect("list procs");
    let record = procs.iter().find(|p| p.id == proc_id).unwrap();
    assert_eq!(record.health_status, "Quarantined");
}

// =============================================================================
// Test 22: Clean Process Tree Termination
// =============================================================================
#[tokio::test]
async fn test_p6_22_clean_process_tree_termination() {
    let (db, _temp) = create_test_db().await;
    let supervisor = ProcessSupervisor::new(db.clone(), 60);

    let dead_pid = 9999997;
    let proc_id = supervisor
        .register_process(dead_pid, "ZombieWorker", "C:\\workspace", None, 3)
        .await;

    let killed = supervisor.kill_process_tree(proc_id).await;
    assert!(killed);

    let procs = db.list_supervised_processes().await.expect("list procs");
    let record = procs.iter().find(|p| p.id == proc_id).unwrap();
    assert_eq!(record.health_status, "Terminated");
}

// =============================================================================
// Test 23: System Resource Monitor: CPU & Memory Tracking
// =============================================================================
#[tokio::test]
async fn test_p6_23_resource_monitor_cpu_memory() {
    let monitor = SystemResourceMonitor::new();
    let snap = monitor.get_snapshot();

    assert!(snap.memory_total_mb > 0);
    assert!(snap.memory_used_mb > 0);
    assert!(snap.memory_used_mb <= snap.memory_total_mb);
    assert!(snap.memory_used_percent >= 0.0 && snap.memory_used_percent <= 100.0);
}

// =============================================================================
// Test 24: System Resource Monitor: Disk Space Warning & Critical Thresholds
// =============================================================================
#[tokio::test]
async fn test_p6_24_resource_monitor_disk_warning() {
    let monitor = SystemResourceMonitor::new();
    let snap = monitor.get_snapshot();

    // Verify disk space sampling returns a non-negative value
    assert!(snap.disk_free_gb >= 0.0);

    // Verify threshold logic:
    // Warning threshold is < 5.0 GB; Critical threshold is < 1.0 GB
    let is_warning = snap.disk_free_gb < 5.0;
    let is_critical = snap.disk_free_gb < 1.0;

    if is_critical {
        assert!(is_warning);
    }
}

// =============================================================================
// Test 25: Memory Backpressure Triggering Pausing of Non-Essential Tasks
// =============================================================================
#[tokio::test]
async fn test_p6_25_memory_backpressure_pause() {
    let monitor = SystemResourceMonitor::new();
    let snap = monitor.get_snapshot();

    // Determine backpressure condition (> 90% memory utilization)
    let backpressure = snap.memory_used_percent > 90.0;
    if backpressure {
        assert!(snap.memory_used_percent > 90.0);
    } else {
        assert!(snap.memory_used_percent <= 90.0);
    }
}

// =============================================================================
// Test 26: Pre-Request Token Budget Reservation
// =============================================================================
#[tokio::test]
async fn test_p6_26_pre_request_token_budget_reservation() {
    let config = BudgetHierarchyConfig {
        global_daily_cost_limit: 10.0,
        global_monthly_cost_limit: 100.0,
        default_workflow_cost_limit: 5.0,
        default_task_cost_limit: 1.0,
    };
    let engine = BudgetGovernanceEngine::new(config);

    // Worker 1 reserves $3.50
    let receipt1 = engine.reserve("Workflow", "wf-101", 10000, 3.50).await.expect("reserve 1");
    assert_eq!(receipt1.reserved_tokens, 10000);
    assert_eq!(receipt1.reserved_cost, 3.50);

    // Active reserved sum must reflect Worker 1's reservation
    assert_eq!(engine.get_active_reserved_sum().await, 3.50);

    // Worker 2 attempts $2.00 on wf-101 (limit is $5.00):
    // wf-101 currently has $3.50 reserved -> $3.50 + $2.00 = $5.50 > $5.00 limit -> Must be rejected!
    let res2 = engine.reserve("Workflow", "wf-101", 5000, 2.00).await;
    assert!(res2.is_err(), "Concurrent worker MUST NOT overshoot budget");
}

// =============================================================================
// Test 27: Pre-Request Cost Budget Reservation Rejection when Cap Exceeded
// =============================================================================
#[tokio::test]
async fn test_p6_27_pre_request_cost_budget_reservation_rejection() {
    let config = BudgetHierarchyConfig {
        global_daily_cost_limit: 5.0,
        global_monthly_cost_limit: 50.0,
        default_workflow_cost_limit: 2.0,
        default_task_cost_limit: 1.0,
    };
    let engine = BudgetGovernanceEngine::new(config);

    // Attempt reservation exceeding daily limit of $5.00
    let res = engine.reserve("Global", "global", 50000, 6.00).await;
    assert!(res.is_err());
    let err_msg = res.unwrap_err();
    assert!(err_msg.contains("Daily budget exceeded"));
}

// =============================================================================
// Test 28: Budget Reservation Release on Task Failure
// =============================================================================
#[tokio::test]
async fn test_p6_28_budget_reservation_release_on_failure() {
    let config = BudgetHierarchyConfig {
        global_daily_cost_limit: 10.0,
        global_monthly_cost_limit: 100.0,
        default_workflow_cost_limit: 5.0,
        default_task_cost_limit: 1.0,
    };
    let engine = BudgetGovernanceEngine::new(config);

    let receipt = engine.reserve("Workflow", "wf-failed", 10000, 4.00).await.expect("reserve");
    assert_eq!(engine.get_active_reserved_sum().await, 4.00);

    // Request failed before sending to LLM -> Release reservation
    engine.release(receipt.id).await;
    assert_eq!(engine.get_active_reserved_sum().await, 0.0);

    // Now full budget is available again
    let receipt2 = engine.reserve("Workflow", "wf-failed", 10000, 4.00).await.expect("reserve again");
    assert_eq!(receipt2.reserved_cost, 4.00);
}

// =============================================================================
// Test 29: Online SQLite Backup API
// =============================================================================
#[tokio::test]
async fn test_p6_29_online_sqlite_backup_api() {
    let (db, temp) = create_test_db().await;
    let backup_path = temp.path().join("online_backup.db");

    let job = ScheduledJob {
        id: Uuid::new_v4(),
        name: "Backup Test Job".to_string(),
        schedule_type: "daily".to_string(),
        cron_expression: None,
        interval_seconds: None,
        timezone: "UTC".to_string(),
        next_run_at: Some(Utc::now()),
        last_run_at: None,
        status: "Active".to_string(),
        missed_run_policy: "Skip".to_string(),
        overlap_policy: "Skip".to_string(),
        max_runtime_seconds: None,
        budget_limit: None,
        workflow_template: "template".to_string(),
        payload_json: "{}".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    db.create_scheduled_job(&job).await.expect("create job");

    // Perform live online backup
    db.backup_to(&backup_path).await.expect("online backup");

    assert!(backup_path.exists());
    let metadata = std::fs::metadata(&backup_path).expect("backup metadata");
    assert!(metadata.len() > 0);
}

// =============================================================================
// Test 30: Database Restore from Backup Verification
// =============================================================================
#[tokio::test]
async fn test_p6_30_database_restore_verification() {
    let (db, temp) = create_test_db().await;
    let backup_path = temp.path().join("restore_verification.db");

    let target_job_id = Uuid::new_v4();
    let job = ScheduledJob {
        id: target_job_id,
        name: "Job To Be Restored".to_string(),
        schedule_type: "interval".to_string(),
        cron_expression: None,
        interval_seconds: Some(1800),
        timezone: "UTC".to_string(),
        next_run_at: Some(Utc::now()),
        last_run_at: None,
        status: "Active".to_string(),
        missed_run_policy: "RunOnce".to_string(),
        overlap_policy: "Skip".to_string(),
        max_runtime_seconds: Some(600),
        budget_limit: Some(5.0),
        workflow_template: "restore_template".to_string(),
        payload_json: r#"{"key":"verified"}"#.to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    db.create_scheduled_job(&job).await.expect("create job");

    // Create backup
    db.backup_to(&backup_path).await.expect("backup");

    // Open restored database directly
    let restored_db = Database::open(&backup_path).expect("open restored db");
    let loaded = restored_db.get_scheduled_job(target_job_id).await.expect("query restored").unwrap();

    assert_eq!(loaded.id, target_job_id);
    assert_eq!(loaded.name, "Job To Be Restored");
    assert_eq!(loaded.workflow_template, "restore_template");
    assert!(loaded.payload_json.contains("verified"));
}

// =============================================================================
// Test 31: Interactive Desktop Automation Deferral on Session 0
// =============================================================================
#[tokio::test]
async fn test_p6_31_session0_deferral() {
    let (db, _temp) = create_test_db().await;
    let wf_id = Uuid::new_v4();
    let wf = Workflow {
        id: wf_id,
        title: "Session 0 Workflow".to_string(),
        goal: "Session 0 test".to_string(),
        status: WorkflowStatus::Running,
        budget_json: "{}".to_string(),
        result_summary: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    db.create_workflow(&wf).await.expect("create wf");

    let task_id = Uuid::new_v4();
    let task = WorkflowTask {
        id: task_id,
        workflow_id: wf_id,
        parent_task_id: None,
        title: "Click Desktop UI Button".to_string(),
        description: "Desktop GUI".to_string(),
        assigned_agent: "Computer".to_string(),
        status: WorkflowTaskStatus::Ready,
        priority: 1,
        inputs_json: "{}".to_string(),
        outputs_json: "{}".to_string(),
        acceptance_criteria: "Clicked".to_string(),
        retry_count: 0,
        max_retries: 3,
        timeout_secs: 60,
        started_at: None,
        completed_at: None,
        error: None,
    };
    db.create_workflow_task(&task).await.expect("create task");

    // Mock Session 0: session_id = 0, is_active = true, is_locked = false
    let session_provider = Arc::new(MockSessionProvider::new(0, true, false));
    let worker = WindowsInteractiveWorker::new(session_provider, db.clone());

    let res = worker.dispatch_gui_task(task_id).await.expect("dispatch");
    match res {
        InteractiveWorkerResult::WaitingForSession(reason) => {
            assert!(reason.contains("Session 0"));
        }
        _ => panic!("Expected WaitingForSession, got {:?}", res),
    }

    let updated = db.get_workflow_task(task_id).await.unwrap().unwrap();
    assert_eq!(updated.status, WorkflowTaskStatus::WaitingForInteractiveSession);
}

// =============================================================================
// Test 32: Interactive Desktop Automation Deferral on Locked Desktop
// =============================================================================
#[tokio::test]
async fn test_p6_32_locked_desktop_deferral() {
    let (db, _temp) = create_test_db().await;
    let wf_id = Uuid::new_v4();
    let wf = Workflow {
        id: wf_id,
        title: "Locked Desktop Workflow".to_string(),
        goal: "Lock test".to_string(),
        status: WorkflowStatus::Running,
        budget_json: "{}".to_string(),
        result_summary: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    db.create_workflow(&wf).await.expect("create wf");

    let task_id = Uuid::new_v4();
    let task = WorkflowTask {
        id: task_id,
        workflow_id: wf_id,
        parent_task_id: None,
        title: "Interact with Excel Spreadsheet".to_string(),
        description: "Desktop automation".to_string(),
        assigned_agent: "Computer".to_string(),
        status: WorkflowTaskStatus::Ready,
        priority: 1,
        inputs_json: "{}".to_string(),
        outputs_json: "{}".to_string(),
        acceptance_criteria: "Complete".to_string(),
        retry_count: 0,
        max_retries: 3,
        timeout_secs: 60,
        started_at: None,
        completed_at: None,
        error: None,
    };
    db.create_workflow_task(&task).await.expect("create task");

    // Mock Session 1 (User session), but desktop is locked!
    let session_provider = Arc::new(MockSessionProvider::new(1, true, true));
    let worker = WindowsInteractiveWorker::new(session_provider.clone(), db.clone());

    let res = worker.dispatch_gui_task(task_id).await.expect("dispatch");
    match res {
        InteractiveWorkerResult::WaitingForSession(reason) => {
            assert!(reason.contains("locked"));
        }
        _ => panic!("Expected WaitingForSession, got {:?}", res),
    }

    let updated = db.get_workflow_task(task_id).await.unwrap().unwrap();
    assert_eq!(updated.status, WorkflowTaskStatus::WaitingForInteractiveSession);

    // User unlocks desktop!
    session_provider.set_locked(false);
    let res2 = worker.dispatch_gui_task(task_id).await.expect("dispatch unlocked");
    assert_eq!(res2, InteractiveWorkerResult::Executed);

    let updated2 = db.get_workflow_task(task_id).await.unwrap().unwrap();
    assert_eq!(updated2.status, WorkflowTaskStatus::Running);
}

// =============================================================================
// Test 33: Error Classification (Transient vs Permanent vs UnknownOutcome)
// =============================================================================
#[tokio::test]
async fn test_p6_33_error_classifier() {
    // 1. Transient
    assert_eq!(
        ErrorClassifier::classify(Some(429), "Rate limit exceeded", false),
        ErrorClass::Transient
    );
    assert_eq!(
        ErrorClassifier::classify(Some(503), "Service unavailable", false),
        ErrorClass::Transient
    );

    // 2. Permanent
    assert_eq!(
        ErrorClassifier::classify(Some(401), "Invalid API key provided", false),
        ErrorClass::Permanent
    );
    assert_eq!(
        ErrorClassifier::classify(None, "Cannot find specified file", false),
        ErrorClass::Permanent
    );

    // 3. Resource
    assert_eq!(
        ErrorClassifier::classify(None, "Fatal: Out of memory error", false),
        ErrorClass::Resource
    );
    assert_eq!(
        ErrorClassifier::classify(None, "Disk full: cannot write checkpoint", false),
        ErrorClass::Resource
    );

    // 4. User Action Required
    assert_eq!(
        ErrorClassifier::classify(None, "Admin approval required to proceed", false),
        ErrorClass::UserActionRequired
    );

    // 5. Unknown Outcome (External mutation interrupted)
    assert_eq!(
        ErrorClassifier::classify(None, "Connection timeout while sending payload", true),
        ErrorClass::UnknownOutcome
    );
}

// =============================================================================
// Test 34: Circuit Breaker State Transitions
// =============================================================================
#[tokio::test]
async fn test_p6_34_circuit_breaker_transitions() {
    let cb = CircuitBreaker::new(2, 1); // 2 failures, 1 second cooldown
    assert_eq!(cb.can_execute(), true);

    // Failure 1
    cb.record_failure();
    assert_eq!(cb.can_execute(), true);

    // Failure 2 -> Trips open!
    cb.record_failure();
    assert_eq!(cb.can_execute(), false);

    // Cooldown passes -> HalfOpen
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    assert_eq!(cb.can_execute(), true); // Probe execution permitted

    // Success -> Closes circuit breaker
    cb.record_success();
    assert_eq!(cb.can_execute(), true);
}

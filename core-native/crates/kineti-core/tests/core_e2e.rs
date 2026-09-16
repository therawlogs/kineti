//! End-to-end integration test verifying the unified agent lifecycle:
//! Pre-Allocation Spend -> HLC -> 20-Entity Provenance Node -> 3-Way Commit Gate ->
//! Actual Spend Settlement -> EBR Snapshot Publish.

use kineti_core::prelude::*;
use std::collections::BTreeMap;

#[test]
fn test_unified_core_lifecycle_e2e() {
    // 1. Initialize core subsystems
    let spend_breaker = SpendCircuitBreaker::default(); // $50 ceiling, $47.50 trip
    let hlc = HybridLogicalClock::new(1);
    let commit_gate = CommitGate::new();
    let snapshot_engine = SnapshotEngine::new(Vec::<NodeId>::new());

    // 2. Pre-allocate spend for Agent Action ($2.50)
    let reservation = spend_breaker
        .reserve(2_500_000)
        .expect("Pre-allocation granted");
    assert_eq!(spend_breaker.reserved_microcents(), 2_500_000);
    assert_eq!(spend_breaker.committed_microcents(), 0);

    // 3. Obtain causal timestamp
    let ts_action = hlc.now().expect("HLC advances");

    // 4. Create root Action node
    let action_entity = KernelEntity::Action(Action {
        id: "act_init".into(),
        task_id: "task_01".into(),
        action_type: "cargo_build".into(),
        description: "Compile kineti-core".into(),
        parameters: json!({"release": true}),
    });

    let action_node = ProvenanceNode::new(
        action_entity,
        vec![],
        ts_action,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("Action node created");

    // 5. Commit root action through gate
    let receipt_act = commit_gate
        .commit(action_node.clone())
        .expect("Action commits to DAG");
    assert_eq!(receipt_act.topological_rank, 0);
    assert_eq!(receipt_act.parent_count, 0);

    // 6. Advance HLC and create child ToolCall node
    let ts_tool = hlc.now().expect("HLC advances for tool");
    assert!(ts_tool > ts_action, "HLC must be strictly monotonic");

    let tool_entity = KernelEntity::ToolCall(ToolCall {
        id: "tool_exec".into(),
        action_id: "act_init".into(),
        tool_name: "run_command".into(),
        arguments: json!({"cmd": "cargo check"}),
        timeout_ms: 10_000,
        sandbox_level: SandboxLevel::WorktreeIsolated,
    });

    let tool_node = ProvenanceNode::new(
        tool_entity,
        vec![action_node.id.clone()],
        ts_tool,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("Tool node created");

    // 7. Commit child tool through gate
    let receipt_tool = commit_gate
        .commit(tool_node.clone())
        .expect("Tool commits to DAG");
    assert_eq!(receipt_tool.topological_rank, 1);
    assert_eq!(receipt_tool.parent_count, 1);

    // 8. Tool completes: settle actual spend ($1.80 actual vs $2.50 estimate)
    let committed_total = spend_breaker
        .commit_with_actual(reservation, 1_800_000)
        .expect("Spend settled");
    assert_eq!(committed_total, 1_800_000);
    assert_eq!(spend_breaker.committed_microcents(), 1_800_000);
    assert_eq!(spend_breaker.reserved_microcents(), 0);
    assert_eq!(spend_breaker.remaining_microcents(), 48_200_000);

    // 9. Atomically publish updated DAG state into snapshot engine
    let mut current_dag = (*snapshot_engine.acquire()).clone();
    current_dag.push(action_node.id);
    current_dag.push(tool_node.id);
    let snap_handle = snapshot_engine.publish(current_dag);

    assert_eq!(snap_handle.version(), 2);
    assert_eq!(snap_handle.len(), 2);
}

#[test]
fn test_multi_agent_dag_branching_and_rollback_e2e() {
    let spend_breaker = SpendCircuitBreaker::default();
    let hlc_agent_1 = HybridLogicalClock::new(101);
    let hlc_agent_2 = HybridLogicalClock::new(102);
    let gate = CommitGate::new();

    // Agent 1 creates Intent
    let res1 = spend_breaker.reserve(1_000_000).unwrap();
    let ts1 = hlc_agent_1.now().unwrap();
    let intent_node = ProvenanceNode::new(
        KernelEntity::Intent(Intent {
            id: "intent_root".into(),
            session_id: "sess_001".into(),
            prompt: "Refactor storage layer".into(),
            raw_input: None,
            embedding_id: None,
        }),
        vec![],
        ts1,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .unwrap();

    let rec_intent = gate.commit(intent_node.clone()).unwrap();
    assert_eq!(rec_intent.topological_rank, 0);
    let _ = spend_breaker.commit(res1).unwrap();

    // Agent 2 reads Intent, advances its HLC past Agent 1's timestamp
    let ts2 = hlc_agent_2.update(&ts1).unwrap();
    assert!(ts2 > ts1);

    // Agent 2 branches: creates Task referencing Intent
    let res2 = spend_breaker.reserve(2_000_000).unwrap();
    let task_node = ProvenanceNode::new(
        KernelEntity::Task(Task {
            id: "task_branch".into(),
            goal_id: "intent_root".into(),
            description: "Implement zero-alloc buffer".into(),
            assigned_actor_id: Some("agent_102".into()),
            deadline_ms: None,
            status: TaskStatus::InProgress,
        }),
        vec![intent_node.id.clone()],
        ts2,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .unwrap();

    let rec_task = gate.commit(task_node.clone()).unwrap();
    assert_eq!(rec_task.topological_rank, 1);
    let _ = spend_breaker.commit(res2).unwrap();

    // Agent 2 registers a RollbackStep (Saga compensation step)
    let ts3 = hlc_agent_2.now().unwrap();
    let rollback_node = ProvenanceNode::new(
        KernelEntity::RollbackStep(RollbackStep {
            id: "rb_step_01".into(),
            task_id: "task_branch".into(),
            step_index: 0,
            compensation_command: "git checkout -- buffer.rs".into(),
            affected_resource: "buffer.rs".into(),
            status: RollbackStatus::Pending,
        }),
        vec![task_node.id.clone()],
        ts3,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .unwrap();

    let rec_rb = gate.commit(rollback_node.clone()).unwrap();
    assert_eq!(rec_rb.topological_rank, 2);
    assert_eq!(gate.count(), 3);
}

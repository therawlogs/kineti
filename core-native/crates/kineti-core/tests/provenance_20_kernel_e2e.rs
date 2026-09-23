//! End-to-end integration test verifying the 20-Entity Universal Provenance Kernel (\Sigma_V)
//! and 13 Canonical Causal Relations (\Sigma_E) committed through the 3-Way Graph Commit Gate.
//!
//! Grounded in Paper 3 (Beyond Vector Search) & Paper 5 (Outcome Engineering)
//! Author: Kineti Research Team (therawlogs.com | Foundational AI Research)

use kineti_core::prelude::*;
use std::collections::BTreeMap;

#[test]
fn test_all_20_provenance_kernel_entities_dag_commit_lifecycle() {
    let gate = CommitGate::new();
    let hlc = HybridLogicalClock::new(42);

    // Verify constant collections
    assert_eq!(EntityType::ALL.len(), 20);
    assert_eq!(RelationType::ALL.len(), 13);

    // 1. Actor
    let ts_actor = hlc.now().unwrap();
    let actor = KernelEntity::Actor(Actor {
        id: "actor_worker_01".into(),
        name: "Context Integrity Worker".into(),
        actor_type: ActorType::Agent,
        public_key: Some("ed25519:test_public_key_bytes".into()),
        role_id: Some("role_engineer".into()),
    });
    assert_eq!(actor.entity_type(), EntityType::Actor);
    let node_actor = ProvenanceNode::new(
        actor,
        vec![],
        ts_actor,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("Actor node created");
    let rec_actor = gate.commit(node_actor.clone()).expect("Actor commits");
    assert_eq!(rec_actor.topological_rank, 0);

    // 2. Role
    let ts_role = hlc.now().unwrap();
    let role = KernelEntity::Role(Role {
        id: "role_engineer".into(),
        name: "Systems Engineer".into(),
        permissions: vec!["code:write".into(), "test:execute".into()],
        max_spend_microcents: 50_000_000,
        trust_level: 95,
    });
    assert_eq!(role.entity_type(), EntityType::Role);
    let node_role = ProvenanceNode::new(
        role,
        vec![node_actor.id.clone()],
        ts_role,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("Role node created");
    let rec_role = gate.commit(node_role.clone()).expect("Role commits");
    assert_eq!(rec_role.topological_rank, 1);

    // 3. Authority
    let ts_auth = hlc.now().unwrap();
    let auth = KernelEntity::Authority(Authority {
        id: "auth_grant_01".into(),
        granter_id: "actor_admin".into(),
        grantee_id: "actor_worker_01".into(),
        scope: vec!["repo:kineti".into()],
        valid_from_ms: 1_000_000,
        valid_until_ms: 2_000_000,
        signature: Some("sig_mock".into()),
    });
    assert_eq!(auth.entity_type(), EntityType::Authority);
    let node_auth = ProvenanceNode::new(
        auth,
        vec![node_role.id.clone()],
        ts_auth,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("Authority node created");
    let rec_auth = gate.commit(node_auth.clone()).expect("Authority commits");
    assert_eq!(rec_auth.topological_rank, 2);

    // 4. Intent
    let ts_intent = hlc.now().unwrap();
    let intent = KernelEntity::Intent(Intent {
        id: "intent_recalibrate".into(),
        session_id: "session_m2".into(),
        prompt: "Recalibrate Context Integrity Layer harness".into(),
        raw_input: None,
        embedding_id: Some("embed_001".into()),
    });
    assert_eq!(intent.entity_type(), EntityType::Intent);
    let node_intent = ProvenanceNode::new(
        intent,
        vec![node_auth.id.clone()],
        ts_intent,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("Intent node created");
    let rec_intent = gate.commit(node_intent.clone()).expect("Intent commits");
    assert_eq!(rec_intent.topological_rank, 3);

    // 5. Goal
    let ts_goal = hlc.now().unwrap();
    let goal = KernelEntity::Goal(Goal {
        id: "goal_align_kernel".into(),
        intent_id: "intent_recalibrate".into(),
        description: "Align 20-entity kernel and DNTI metrics".into(),
        success_criteria: vec!["cargo test passes".into(), "bun test passes".into()],
        status: GoalStatus::Active,
    });
    assert_eq!(goal.entity_type(), EntityType::Goal);
    let node_goal = ProvenanceNode::new(
        goal,
        vec![node_intent.id.clone()],
        ts_goal,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("Goal node created");
    let rec_goal = gate.commit(node_goal.clone()).expect("Goal commits");
    assert_eq!(rec_goal.topological_rank, 4);

    // 6. Task
    let ts_task = hlc.now().unwrap();
    let task = KernelEntity::Task(Task {
        id: "task_implement_dnti".into(),
        goal_id: "goal_align_kernel".into(),
        description: "Implement dnti.rs in kineti-harness".into(),
        assigned_actor_id: Some("actor_worker_01".into()),
        deadline_ms: Some(1_800_000_000),
        status: TaskStatus::InProgress,
    });
    assert_eq!(task.entity_type(), EntityType::Task);
    let node_task = ProvenanceNode::new(
        task,
        vec![node_goal.id.clone()],
        ts_task,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("Task node created");
    let rec_task = gate.commit(node_task.clone()).expect("Task commits");
    assert_eq!(rec_task.topological_rank, 5);

    // 7. Constraint
    let ts_constraint = hlc.now().unwrap();
    let constraint = KernelEntity::Constraint(Constraint {
        id: "constraint_spend_ceiling".into(),
        name: "Spend Hardware Ceiling".into(),
        expression: "spend <= 50.00".into(),
        enforcement_level: EnforcementLevel::Strict,
        is_active: true,
    });
    assert_eq!(constraint.entity_type(), EntityType::Constraint);
    let node_constraint = ProvenanceNode::new(
        constraint,
        vec![node_task.id.clone()],
        ts_constraint,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("Constraint node created");
    let rec_constraint = gate.commit(node_constraint.clone()).expect("Constraint commits");
    assert_eq!(rec_constraint.topological_rank, 6);

    // 8. Decision
    let ts_decision = hlc.now().unwrap();
    let decision = KernelEntity::Decision(Decision {
        id: "dec_dnti_formulation".into(),
        task_id: Some("task_implement_dnti".into()),
        chosen_option: "Paper 5 Three-Factor Formula".into(),
        rejected_options: vec!["Raw Accuracy".into(), "Unweighted Delta".into()],
        rationale: "Loss-averse verification prevents Goodhart gaming".into(),
    });
    assert_eq!(decision.entity_type(), EntityType::Decision);
    let node_decision = ProvenanceNode::new(
        decision,
        vec![node_constraint.id.clone()],
        ts_decision,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("Decision node created");
    let rec_decision = gate.commit(node_decision.clone()).expect("Decision commits");
    assert_eq!(rec_decision.topological_rank, 7);

    // 9. Dependency
    let ts_dep = hlc.now().unwrap();
    let dep = KernelEntity::Dependency(Dependency {
        id: "dep_kineti_core_to_harness".into(),
        source_node_id: "node_harness".into(),
        target_node_id: "node_core".into(),
        dependency_type: DependencyType::Causal,
    });
    assert_eq!(dep.entity_type(), EntityType::Dependency);
    let node_dep = ProvenanceNode::new(
        dep,
        vec![node_decision.id.clone()],
        ts_dep,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("Dependency node created");
    let rec_dep = gate.commit(node_dep.clone()).expect("Dependency commits");
    assert_eq!(rec_dep.topological_rank, 8);

    // 10. Action
    let ts_action = hlc.now().unwrap();
    let action = KernelEntity::Action(Action {
        id: "act_write_dnti".into(),
        task_id: "task_implement_dnti".into(),
        action_type: "file_write".into(),
        description: "Write dnti.rs implementation".into(),
        parameters: json!({"path": "core-native/crates/kineti-harness/src/dnti.rs"}),
    });
    assert_eq!(action.entity_type(), EntityType::Action);
    let node_action = ProvenanceNode::new(
        action,
        vec![node_dep.id.clone()],
        ts_action,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("Action node created");
    let rec_action = gate.commit(node_action.clone()).expect("Action commits");
    assert_eq!(rec_action.topological_rank, 9);

    // 11. ToolCall
    let ts_tool = hlc.now().unwrap();
    let tool = KernelEntity::ToolCall(ToolCall {
        id: "tool_cargo_test".into(),
        action_id: "act_write_dnti".into(),
        tool_name: "run_command".into(),
        arguments: json!({"command": "cargo test"}),
        timeout_ms: 30_000,
        sandbox_level: SandboxLevel::WorktreeIsolated,
    });
    assert_eq!(tool.entity_type(), EntityType::ToolCall);
    let node_tool = ProvenanceNode::new(
        tool,
        vec![node_action.id.clone()],
        ts_tool,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("ToolCall node created");
    let rec_tool = gate.commit(node_tool.clone()).expect("ToolCall commits");
    assert_eq!(rec_tool.topological_rank, 10);

    // 12. RollbackStep
    let ts_rb = hlc.now().unwrap();
    let rollback = KernelEntity::RollbackStep(RollbackStep {
        id: "rb_step_revert_dnti".into(),
        task_id: "task_implement_dnti".into(),
        step_index: 0,
        compensation_command: "rm -f core-native/crates/kineti-harness/src/dnti.rs".into(),
        affected_resource: "dnti.rs".into(),
        status: RollbackStatus::Pending,
    });
    assert_eq!(rollback.entity_type(), EntityType::RollbackStep);
    let node_rb = ProvenanceNode::new(
        rollback,
        vec![node_tool.id.clone()],
        ts_rb,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("RollbackStep node created");
    let rec_rb = gate.commit(node_rb.clone()).expect("RollbackStep commits");
    assert_eq!(rec_rb.topological_rank, 11);

    // 13. Observation
    let ts_obs = hlc.now().unwrap();
    let obs = KernelEntity::Observation(Observation {
        id: "obs_test_results".into(),
        action_id: "act_write_dnti".into(),
        stdout: Some("test result: ok. 10 passed".into()),
        stderr: None,
        exit_code: Some(0),
        latency_ms: 125,
    });
    assert_eq!(obs.entity_type(), EntityType::Observation);
    let node_obs = ProvenanceNode::new(
        obs,
        vec![node_rb.id.clone()],
        ts_obs,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("Observation node created");
    let rec_obs = gate.commit(node_obs.clone()).expect("Observation commits");
    assert_eq!(rec_obs.topological_rank, 12);

    // 14. Evidence
    let ts_ev = hlc.now().unwrap();
    let ev = KernelEntity::Evidence(Evidence {
        id: "ev_cargo_receipt".into(),
        observation_id: Some("obs_test_results".into()),
        evidence_type: "cargo_test_exit_zero".into(),
        digest: "blake3:c0ffee1234567890abcdef".into(),
        verification_method: "kineti-evidence-sha256".into(),
    });
    assert_eq!(ev.entity_type(), EntityType::Evidence);
    let node_ev = ProvenanceNode::new(
        ev,
        vec![node_obs.id.clone()],
        ts_ev,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("Evidence node created");
    let rec_ev = gate.commit(node_ev.clone()).expect("Evidence commits");
    assert_eq!(rec_ev.topological_rank, 13);

    // 15. StateChange
    let ts_sc = hlc.now().unwrap();
    let sc = KernelEntity::StateChange(StateChange {
        id: "sc_dnti_rs_created".into(),
        entity_ref: "file:dnti.rs".into(),
        before_hash: "blake3:000000000000000000000000".into(),
        after_hash: "blake3:111111111111111111111111".into(),
        path: "core-native/crates/kineti-harness/src/dnti.rs".into(),
        patch: Some("+pub fn calculate_dnti()".into()),
    });
    assert_eq!(sc.entity_type(), EntityType::StateChange);
    let node_sc = ProvenanceNode::new(
        sc,
        vec![node_ev.id.clone()],
        ts_sc,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("StateChange node created");
    let rec_sc = gate.commit(node_sc.clone()).expect("StateChange commits");
    assert_eq!(rec_sc.topological_rank, 14);

    // 16. Metric
    let ts_metric = hlc.now().unwrap();
    let metric = KernelEntity::Metric(Metric {
        id: "metric_dnti_score".into(),
        metric_name: "dnti_verified_impact".into(),
        value: 1.42,
        unit: "impact_score".into(),
        spend_microcents: Some(150_000), // $0.15 spend
    });
    assert_eq!(metric.entity_type(), EntityType::Metric);
    let node_metric = ProvenanceNode::new(
        metric,
        vec![node_sc.id.clone()],
        ts_metric,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("Metric node created");
    let rec_metric = gate.commit(node_metric.clone()).expect("Metric commits");
    assert_eq!(rec_metric.topological_rank, 15);

    // 17. Exception
    let ts_ex = hlc.now().unwrap();
    let ex = KernelEntity::Exception(Exception {
        id: "ex_handled_lint".into(),
        error_code: "UNUSED_IMPORT".into(),
        message: "Non-fatal unused import warning resolved".into(),
        stack_trace: None,
        fatal: false,
    });
    assert_eq!(ex.entity_type(), EntityType::Exception);
    let node_ex = ProvenanceNode::new(
        ex,
        vec![node_metric.id.clone()],
        ts_ex,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("Exception node created");
    let rec_ex = gate.commit(node_ex.clone()).expect("Exception commits");
    assert_eq!(rec_ex.topological_rank, 16);

    // 18. Approval
    let ts_app = hlc.now().unwrap();
    let app = KernelEntity::Approval(Approval {
        id: "app_code_review".into(),
        task_id: "task_implement_dnti".into(),
        approver_id: "actor_admin".into(),
        approved: true,
        reason: Some("Passed all 20 kernel tests".into()),
        signature: Some("sig_approval_receipt".into()),
    });
    assert_eq!(app.entity_type(), EntityType::Approval);
    let node_app = ProvenanceNode::new(
        app,
        vec![node_ex.id.clone()],
        ts_app,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("Approval node created");
    let rec_app = gate.commit(node_app.clone()).expect("Approval commits");
    assert_eq!(rec_app.topological_rank, 17);

    // 19. ReviewRequired
    let ts_rr = hlc.now().unwrap();
    let rr = KernelEntity::ReviewRequired(ReviewRequired {
        id: "rr_security_signoff".into(),
        reason: "Cryptographic ticket dual signature check".into(),
        required_role: "SecurityAuditor".into(),
        urgency: ReviewUrgency::Medium,
        status: ReviewStatus::Approved,
    });
    assert_eq!(rr.entity_type(), EntityType::ReviewRequired);
    let node_rr = ProvenanceNode::new(
        rr,
        vec![node_app.id.clone()],
        ts_rr,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("ReviewRequired node created");
    let rec_rr = gate.commit(node_rr.clone()).expect("ReviewRequired commits");
    assert_eq!(rec_rr.topological_rank, 18);

    // 20. Outcome
    let ts_outcome = hlc.now().unwrap();
    let outcome = KernelEntity::Outcome(Outcome {
        id: "outcome_m2_verified".into(),
        goal_id: "goal_align_kernel".into(),
        status: OutcomeStatus::Success,
        ovt_ticket: Some("ovt_ticket_dual_signed_hash_12345".into()),
        verified_at_ms: Some(1_750_000_000),
    });
    assert_eq!(outcome.entity_type(), EntityType::Outcome);
    let node_outcome = ProvenanceNode::new(
        outcome,
        vec![node_rr.id.clone()],
        ts_outcome,
        BTreeMap::new(),
        HashAlgorithm::Blake3,
    )
    .expect("Outcome node created");
    let rec_outcome = gate.commit(node_outcome.clone()).expect("Outcome commits");
    assert_eq!(rec_outcome.topological_rank, 19);

    // Final DAG count verification: exactly 20 nodes committed
    assert_eq!(gate.count(), 20);

    // Verify all 20 nodes pass content address integrity checks
    let nodes = [
        &node_actor, &node_role, &node_auth, &node_intent, &node_goal,
        &node_task, &node_constraint, &node_decision, &node_dep, &node_action,
        &node_tool, &node_rb, &node_obs, &node_ev, &node_sc,
        &node_metric, &node_ex, &node_app, &node_rr, &node_outcome,
    ];
    for (i, n) in nodes.iter().enumerate() {
        assert!(n.verify_content_address().is_ok(), "Node #{} failed hash check", i + 1);
    }
}

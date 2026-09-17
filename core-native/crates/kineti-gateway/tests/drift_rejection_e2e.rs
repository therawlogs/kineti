//! End-to-end integration tests for multi-step anti-drift rejection in the Kineti Gateway.
//!
//! Verifies that:
//! 1. Verbatim root goals are locked and preserved across multiple execution steps.
//! 2. Multi-step "telephone game" drift is rejected: successive actions cannot silently mutate the root ask.
//! 3. Budget overruns (even 10%) are blocked with clean, anchored explanations.
//! 4. Counterparty drift (e.g. ScalperHub instead of Ticketmaster) is blocked.
//! 5. Friction triage ladder appropriately retries noise, silently reroutes broken surfaces, and cleanly escalates real constraints.

use kineti_connectors::{ActionAuthorizationToken, Value};
use kineti_core::conversation::UserTier;
use kineti_core::current_epoch_millis;
use kineti_core::root_goal::{
    DriftEvaluation, FrictionResolution, FrictionType,
};
use kineti_core::spend::UserSpendQuota;
use kineti_gateway::router::{
    EventSource, GatewayRouter, IncomingStimulusEvent, OutboundReply,
};
use std::collections::BTreeMap;

#[test]
fn test_e2e_multi_step_drift_rejection_and_telephone_game_prevention() {
    let router = GatewayRouter::new();
    let user_id = "user_vip_whatsapp_99";
    let quota = UserSpendQuota::new(user_id, UserTier::Pro.daily_quota_microcents());
    let now_ms = current_epoch_millis();

    // -------------------------------------------------------------------
    // STEP 0: Lock Verbatim Root Goal via WhatsApp Webhook
    // -------------------------------------------------------------------
    let lock_stimulus = IncomingStimulusEvent::webhook(
        user_id,
        "Goal: Book 2 floor seats for Hans Zimmer at MSG under $350 on Ticketmaster",
        None,
    );
    let lock_receipt = router.dispatch_event(lock_stimulus, &quota);
    assert_eq!(lock_receipt.source, EventSource::Webhook);
    assert_eq!(lock_receipt.drift_evaluation, Some(DriftEvaluation::InBounds));

    let active_goal = router
        .get_active_root_goal(user_id)
        .expect("Expected active root goal to be registered");
    assert_eq!(active_goal.boundaries.max_spend_microcents, Some(350_000_000));
    assert_eq!(
        active_goal.boundaries.counterparty.as_deref(),
        Some("Ticketmaster")
    );
    assert_eq!(
        active_goal.raw_user_ask,
        "Book 2 floor seats for Hans Zimmer at MSG under $350 on Ticketmaster"
    );

    // -------------------------------------------------------------------
    // STEP 1: In-Bounds Action (Search & Execution within Boundaries)
    // -------------------------------------------------------------------
    let mut step1_payload = BTreeMap::new();
    step1_payload.insert("spend_microcents".to_string(), Value::from(330_000_000u64));
    step1_payload.insert(
        "counterparty".to_string(),
        Value::String("Ticketmaster".to_string()),
    );
    let token1 = ActionAuthorizationToken::mint(
        "token_step_1",
        user_id,
        "financial_checkout",
        "execute_purchase",
        &Value::Object(step1_payload.clone()),
        600,
        now_ms,
    );
    let step1_event = IncomingStimulusEvent::webhook(user_id, "Authorize step 1 purchase", None)
        .with_action("execute_purchase", Value::Object(step1_payload))
        .with_token(token1);
    let step1_receipt = router.dispatch_event(step1_event, &quota);

    assert!(step1_receipt.action_gate_passed);
    assert_eq!(step1_receipt.drift_evaluation, Some(DriftEvaluation::InBounds));
    if let OutboundReply::Text { body } = step1_receipt.reply {
        assert!(body.contains("executed successfully"));
    } else {
        panic!("Expected successful purchase confirmation in step 1");
    }

    // -------------------------------------------------------------------
    // STEP 2: The Multi-Step "Telephone Game" Test
    // Intermediate agent proposes: "Accept 10% more to guarantee seats"
    // Drift inspector evaluates against the IMMUTABLE ROOT ASK, not Step 1!
    // -------------------------------------------------------------------
    let mut step2_payload = BTreeMap::new();
    step2_payload.insert("spend_microcents".to_string(), Value::from(385_000_000u64)); // $385 = 10% overrun of $350
    step2_payload.insert(
        "counterparty".to_string(),
        Value::String("Ticketmaster".to_string()),
    );
    let token2 = ActionAuthorizationToken::mint(
        "token_step_2",
        user_id,
        "financial_checkout",
        "execute_purchase",
        &Value::Object(step2_payload.clone()),
        600,
        now_ms,
    );
    let step2_event = IncomingStimulusEvent::webhook(
        user_id,
        "Attempt to accept 10% more to guarantee floor seats",
        None,
    )
    .with_action("execute_purchase", Value::Object(step2_payload))
    .with_token(token2);
    let step2_receipt = router.dispatch_event(step2_event, &quota);

    // MUST FAIL CLOSED: Drift engine rejects overrun
    assert!(!step2_receipt.action_gate_passed);
    match step2_receipt.drift_evaluation {
        Some(DriftEvaluation::GoalMutationBlocked {
            ref reason,
            ref verbatim_ask,
        }) => {
            assert!(
                reason.contains("exceeds user ceiling of $350.00")
                    || reason.contains("compromise user terms"),
                "Expected overrun reason, got: {}",
                reason
            );
            assert_eq!(
                verbatim_ask,
                "Book 2 floor seats for Hans Zimmer at MSG under $350 on Ticketmaster"
            );
        }
        other => panic!("Expected GoalMutationBlocked, got: {:?}", other),
    }

    // -------------------------------------------------------------------
    // STEP 3: Counterparty Drift Rejection
    // Third-party API fails on Ticketmaster; agent suggests buying from "StubHub"
    // -------------------------------------------------------------------
    let mut step3_payload = BTreeMap::new();
    step3_payload.insert("spend_microcents".to_string(), Value::from(320_000_000u64)); // Price is within budget!
    step3_payload.insert(
        "counterparty".to_string(),
        Value::String("StubHub".to_string()), // Unauthorized counterparty!
    );
    let token3 = ActionAuthorizationToken::mint(
        "token_step_3",
        user_id,
        "financial_checkout",
        "execute_purchase",
        &Value::Object(step3_payload.clone()),
        600,
        now_ms,
    );
    let step3_event = IncomingStimulusEvent::webhook(
        user_id,
        "Attempt purchase from alternative vendor StubHub",
        None,
    )
    .with_action("execute_purchase", Value::Object(step3_payload))
    .with_token(token3);
    let step3_receipt = router.dispatch_event(step3_event, &quota);

    // MUST FAIL CLOSED: Unauthorized counterparty blocked
    assert!(!step3_receipt.action_gate_passed);
    match step3_receipt.drift_evaluation {
        Some(DriftEvaluation::GoalMutationBlocked {
            ref reason,
            ref verbatim_ask,
        }) => {
            assert!(
                reason.contains("does not match authorized counterparty 'Ticketmaster'"),
                "Expected counterparty mismatch reason, got: {}",
                reason
            );
            assert_eq!(
                verbatim_ask,
                "Book 2 floor seats for Hans Zimmer at MSG under $350 on Ticketmaster"
            );
        }
        other => panic!("Expected GoalMutationBlocked for counterparty, got: {:?}", other),
    }

    // -------------------------------------------------------------------
    // STEP 4: Friction Triage Ladder in Autonomous Recovery
    // -------------------------------------------------------------------
    // 4A: Transient network timeout (Noise) -> Silent retry
    let noise = FrictionType::Noise {
        message: "504 Gateway Timeout on Ticketmaster SeatMap".to_string(),
    };
    let resolution_noise = router.triage_action_friction(user_id, noise, 5_000);
    assert_eq!(
        resolution_noise,
        FrictionResolution::RetryWithBackoff { delay_ms: 1000 }
    );

    // 4B: Broken surface with valid alternative -> Silent reroute
    let broken_surface = FrictionType::BrokenSurface {
        surface_name: "Ticketmaster Mobile API".to_string(),
        alternative_surface: Some("Ticketmaster Partner XML Gateway".to_string()),
    };
    let resolution_surface = router.triage_action_friction(user_id, broken_surface, 15_000);
    match resolution_surface {
        FrictionResolution::SilentReroute {
            target_surface,
            verbatim_goal,
        } => {
            assert_eq!(target_surface, "Ticketmaster Partner XML Gateway");
            assert!(verbatim_goal.contains("Book 2 floor seats for Hans Zimmer"));
        }
        other => panic!("Expected SilentReroute, got: {:?}", other),
    }

    // 4C: Real Constraint (Seller refuses terms) -> Escalate Clean No
    let real_constraint = FrictionType::RealConstraint {
        obstacle: "All Ticketmaster inventory sold out; scalper prices start at $490".to_string(),
        mutates_done_definition: true,
    };
    let resolution_constraint = router.triage_action_friction(user_id, real_constraint, 30_000);
    match resolution_constraint {
        FrictionResolution::EscalateCleanNo {
            clean_no_reason,
            banked_data,
        } => {
            assert!(clean_no_reason.contains("All Ticketmaster inventory sold out"));
            assert!(clean_no_reason.contains("Book 2 floor seats for Hans Zimmer"));
            assert!(banked_data.contains("zero goal mutation permitted"));
        }
        other => panic!("Expected EscalateCleanNo, got: {:?}", other),
    }

    // -------------------------------------------------------------------
    // STEP 5: Root Goal Invariance via iMessage Stimulus
    // -------------------------------------------------------------------
    let imessage_stimulus = IncomingStimulusEvent::imessage(
        user_id,
        "What is the status of my Hans Zimmer tickets?",
        None,
    );
    let imessage_receipt = router.dispatch_event(imessage_stimulus, &quota);
    assert_eq!(imessage_receipt.source, EventSource::Webhook);
    // Root goal is still intact and active
    let current_goal = router.get_active_root_goal(user_id).unwrap();
    assert_eq!(
        current_goal.raw_user_ask,
        "Book 2 floor seats for Hans Zimmer at MSG under $350 on Ticketmaster"
    );
}

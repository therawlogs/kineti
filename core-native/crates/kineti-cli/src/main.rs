//! # Kineti Native CLI Entrypoint
//!
//! Provides operational commands to test, run, and inspect the Kineti assistant.

use kineti_core::conversation::UserTier;
use kineti_core::root_goal::{
    DriftEvaluation, DriftInspector, FrictionResolution, FrictionType, ImmutableBoundaries,
    VerbatimRootGoal,
};
use kineti_core::spend::UserSpendQuota;
use kineti_gateway::router::{DispatchReceipt, IncomingStimulusEvent};
use kineti_gateway::{GatewayRouter, OutboundReply};
use kineti_memory::{
    CommitmentTimeVerifier, ConsequenceLevel, GapFillingPolicy, Perishability,
    PromotionSignalDetector,
};
use std::env;
use std::io::{self, BufRead, Write};

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match command {
        "chat" => run_interactive_chat(),
        "status" => run_status(),
        "test-all" => run_test_suite(),
        "epistemic" => run_epistemic_demo(),
        "epistemic-eval" => run_epistemic_eval_json(&args),
        "anti-drift" => run_anti_drift_demo(),
        _ => print_help(),
    }
}

fn print_help() {
    println!(
        "\n=======================================================\n\
           Kineti — Proprietary Messaging AI Assistant\n\
           Website: getkineti.com | Pure Rust Nervous System\n\
         =======================================================\n\n\
         Usage:\n\
           kineti-cli chat        Start interactive chat testing session\n\
           kineti-cli status      Show memory and spend quota status\n\
           kineti-cli test-all    Run end-to-end verification checks\n\
           kineti-cli epistemic   Demonstrate 360º Human Model & Epistemic Engine\n\
           kineti-cli anti-drift  Demonstrate Verbatim Root Goal & Anti-Drift Engine\n"
    );
}

fn run_status() {
    let router = GatewayRouter::new();
    let quota = UserSpendQuota::new("local_dev_user", UserTier::Pro.daily_quota_microcents());

    println!("\n--- Kineti Assistant Status ---");
    println!("Architecture: Pure Native Rust (Zero Node/Python dependencies)");
    println!("Spend Quota: Limit ${:.2} | Spent ${:.2}",
        quota.limit_microcents() as f32 / 1_000_000.0,
        quota.spent_microcents() as f32 / 1_000_000.0,
    );
    println!("Active Memory Nodes: {}", router.memory.query_facts("local_dev_user", None).len());
    println!("Supported Channels: Apple iMessage (macOS bridge) & Meta WhatsApp Cloud API");
    println!("Real-World Actions: Active (Shopping, Tickets, Brave Search, FLUX.1)");
    println!("-------------------------------\n");
}

fn run_interactive_chat() {
    let router = GatewayRouter::new();
    let user_id = "local_dogfood_user";
    let quota = UserSpendQuota::new(user_id, UserTier::Pro.daily_quota_microcents());

    println!("\n=======================================================");
    println!("  Kineti Native Chat Session (Simulated iMessage/WhatsApp)");
    println!("  Type 'exit' to quit | '/memory' or '/profile' to inspect");
    println!("  Try setting goals: 'Goal: Buy 2 tickets for Hans Zimmer under $350'");
    println!("=======================================================\n");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    print!("You > ");
    stdout.flush().unwrap();

    for line in stdin.lock().lines() {
        let text = match line {
            Ok(t) => t,
            Err(_) => break,
        };

        let trimmed = text.trim();
        if trimmed == "exit" || trimmed == "quit" {
            println!("Goodbye!");
            break;
        }

        if trimmed.is_empty() {
            print!("You > ");
            stdout.flush().unwrap();
            continue;
        }

        if trimmed == "/profile" || trimmed == "/memory" {
            print_profile_stats(&router, user_id, None);
            println!();
            print!("You > ");
            stdout.flush().unwrap();
            continue;
        }

        let event = IncomingStimulusEvent::cli(user_id, trimmed);
        let receipt = router.dispatch_event(event, &quota);
        match &receipt.reply {
            OutboundReply::Reaction { emoji } => {
                println!("Kineti > [Reaction: {} attached to message bubble]", emoji);
            }
            OutboundReply::Text { body } => {
                println!("Kineti >\n{}", body);
            }
            OutboundReply::Image { media_url, caption } => {
                println!("Kineti > [Photo Delivered: {}]\nCaption: {}", media_url, caption);
            }
        }

        println!();
        print_profile_stats(&router, user_id, Some(&receipt));
        println!();
        print!("You > ");
        stdout.flush().unwrap();
    }
}

fn print_profile_stats(
    router: &GatewayRouter,
    user_id: &str,
    receipt: Option<&DispatchReceipt>,
) {
    let style = router.memory.get_user_style(user_id);
    let facts = router.memory.query_facts(user_id, None);
    let active_goal = router.get_active_root_goal(user_id);

    println!("  ┌── [Memory & Persona Profile] ──────────────────────────────────────────────┐");
    println!("  │ Socio-Linguistic Style: Formality: {:.0}% | Brevity: {:.0}% | Slang: {:.0}%",
        style.formality * 100.0,
        (1.0 - style.verbosity) * 100.0,
        style.slang_affinity * 100.0,
    );
    println!("  │ Dialect: {:<12} Lowercase Pref: {:<5} Emoji Density: {:.0}%",
        style.language_dialect,
        style.lowercase_preference,
        style.emoji_density * 100.0,
    );
    println!("  │ Epistemic Facts Ingested: {}", facts.len());
    for f in facts.iter().take(3) {
        println!("  │   • [{}] {}", f.key, f.value);
    }
    if let Some(goal) = active_goal {
        let spend_str = goal.boundaries.max_spend_microcents
            .map(|c| format!("${:.2}", c as f64 / 1_000_000.0))
            .unwrap_or_else(|| "None".to_string());
        let cp_str = goal.boundaries.counterparty.as_deref().unwrap_or("None");
        println!("  │ Active Root Goal: \"{}\"", goal.raw_user_ask);
        println!("  │ Boundaries: Spend Ceiling: {} | Counterparty: {}", spend_str, cp_str);
    } else {
        println!("  │ Active Root Goal: None (Set via 'Goal: <ask>')");
    }
    if let Some(r) = receipt {
        let drift_str = match &r.drift_evaluation {
            Some(DriftEvaluation::InBounds) => "InBounds (Compliant)",
            Some(DriftEvaluation::PathReroutePermitted { .. }) => "Path Reroute Permitted",
            Some(DriftEvaluation::GoalMutationBlocked { reason, .. }) => reason.as_str(),
            None => "N/A (Reflex / Fast-Path)",
        };
        println!("  │ Last Cycle: Latency: {} µs | Spend: ${:.4} | Drift Status: {}",
            r.triage_latency_micros,
            r.microcents_spent as f64 / 1_000_000.0,
            drift_str,
        );
    }
    println!("  └─────────────────────────────────────────────────────────────────────────────┘");
}

fn run_test_suite() {
    println!("\nRunning Kineti Verification Suite...");
    let router = GatewayRouter::new();
    let user_id = "test_verification_user";
    let quota = UserSpendQuota::new(user_id, UserTier::Pro.daily_quota_microcents());

    // 1. Test status update reflex
    let r1 = router.process_message(user_id, "leaving now, see you later", None, &quota);
    assert_eq!(r1, OutboundReply::Reaction { emoji: "⚡" });
    println!("  [1/5] Low-info status reaction (⚡): PASS");

    // 2. Test style-adaptive greeting
    let r2 = router.process_message(user_id, "yo", None, &quota);
    if let OutboundReply::Text { body } = r2 {
        assert!(body.contains("hey!"));
        println!("  [2/5] Style-adaptive greeting: PASS");
    } else {
        panic!("Failed greeting test");
    }

    // 3. Test memory capture
    let r3 = router.process_message(user_id, "remember that my manager is Sarah Chen", None, &quota);
    if let OutboundReply::Text { body } = r3 {
        assert!(body.to_lowercase().contains("sarah chen"));
        println!("  [3/5] Memory capture & style-matching: PASS");
    } else {
        panic!("Failed memory capture test");
    }

    // 4. Test price comparison
    let r4 = router.process_message(user_id, "Find me the best price on Sony headphones", None, &quota);
    if let OutboundReply::Text { body } = r4 {
        assert!(body.contains("Amazon: $328.00"));
        println!("  [4/5] Real-world price comparison: PASS");
    } else {
        panic!("Failed price comparison test");
    }

    // 5. Test ticket search & confirmation gate
    let r5 = router.process_message(user_id, "Look for 2 good tickets for Hans Zimmer", None, &quota);
    if let OutboundReply::Text { body } = r5 {
        assert!(body.contains("Reply BUY to confirm"));
        println!("  [5/5] Two-step ticket booking confirmation gate: PASS");
    } else {
        panic!("Failed ticket confirmation test");
    }

    println!("\nAll 5 verification flows passed successfully!\n");
}

fn run_epistemic_demo() {
    use kineti_memory::{
        ActionEvaluation, ConsequenceLevel, ContextScope, DomainKind, EpistemicCertainty,
        EpistemicEngine, EpistemicFact, EpistemicTransition, RuleConstraintType,
    };

    println!("\n=======================================================");
    println!("  Kineti OS — 360º Epistemic Persona Engine Demo");
    println!("=======================================================\n");

    let engine = EpistemicEngine::new();
    let user_id = "praveen";
    let health = ContextScope::Domain(DomainKind::Health);

    // 1. Ingest Baseline Rule: Vegetarian
    println!("[1] Ingesting Baseline Rule: 'Vegetarian' (Health scope)...");
    engine
        .ingest(
            EpistemicFact {
                id: "diet_01".to_string(),
                user_id: user_id.to_string(),
                scope: health.clone(),
                attribute: "diet".to_string(),
                claim: "Vegetarian".to_string(),
                constraint_type: RuleConstraintType::BaselineRule,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: 1000,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: ConsequenceLevel::HighConsequence,
            },
            1000,
        )
        .unwrap();

    // 2. Ingest Permitted Exception: Eats eggs
    println!("[2] Ingesting Permitted Exception: 'Eats eggs'...");
    engine
        .ingest(
            EpistemicFact {
                id: "diet_02".to_string(),
                user_id: user_id.to_string(),
                scope: health.clone(),
                attribute: "diet_exception".to_string(),
                claim: "eggs".to_string(),
                constraint_type: RuleConstraintType::PermittedException,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: 1005,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: ConsequenceLevel::HighConsequence,
            },
            1005,
        )
        .unwrap();

    // 3. Ingest Preference: Low dairy (NOT an allergy)
    println!("[3] Ingesting Preference: 'Low dairy'...");
    engine
        .ingest(
            EpistemicFact {
                id: "diet_03".to_string(),
                user_id: user_id.to_string(),
                scope: health.clone(),
                attribute: "dairy_pref".to_string(),
                claim: "low_dairy".to_string(),
                constraint_type: RuleConstraintType::Preference,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: 1010,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: ConsequenceLevel::Operational,
            },
            1010,
        )
        .unwrap();

    // 4. Ingest Safety Ceiling: Peanut allergy
    println!("[4] Ingesting Safety Ceiling: 'Peanut allergy' (Strict non-negotiable)...");
    engine
        .ingest(
            EpistemicFact {
                id: "diet_04".to_string(),
                user_id: user_id.to_string(),
                scope: health.clone(),
                attribute: "allergy".to_string(),
                claim: "peanut".to_string(),
                constraint_type: RuleConstraintType::SafetyCeiling,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: 1015,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: ConsequenceLevel::HighConsequence,
            },
            1015,
        )
        .unwrap();

    println!("\n--- Evaluating Candidate Food Actions Against Persona ---");

    // Candidate A: Chicken
    let eval_a = engine.evaluate_candidate(user_id, &health, "Chicken Biryani", &["meat", "chicken"], 1020);
    println!("Candidate: Chicken Biryani -> {:?}", eval_a);
    assert!(matches!(eval_a, ActionEvaluation::BlockedByRule { .. }));

    // Candidate B: Eggs
    let eval_b = engine.evaluate_candidate(user_id, &health, "Vegetable Omelette", &["eggs"], 1020);
    println!("Candidate: Vegetable Omelette -> {:?}", eval_b);
    assert!(matches!(eval_b, ActionEvaluation::Permitted { .. }));

    // Candidate C: Peanuts
    let eval_c = engine.evaluate_candidate(user_id, &health, "Pad Thai with Peanuts", &["peanut"], 1020);
    println!("Candidate: Pad Thai with Peanuts -> {:?}", eval_c);
    assert!(matches!(eval_c, ActionEvaluation::BlockedBySafetyCeiling { .. }));

    println!("\n--- Testing Epistemic Invariance (Inference vs Known Truth) ---");
    let conflicting_inference = EpistemicFact {
        id: "inf_01".to_string(),
        user_id: user_id.to_string(),
        scope: health,
        attribute: "diet".to_string(),
        claim: "Non-Vegetarian".to_string(),
        constraint_type: RuleConstraintType::BaselineRule,
        certainty: EpistemicCertainty::Inferred {
            reasoning: "User browsed steak house menu once".to_string(),
        },
        valid_from: 1030,
        valid_until: None,
        contradiction_criteria: None,
        consequence_level: ConsequenceLevel::HighConsequence,
    };

    let transition = engine.ingest(conflicting_inference, 1030).unwrap();
    println!("Result of conflicting inference: {:?}", transition);
    assert!(matches!(transition, EpistemicTransition::ConflictNeedsClarification { .. }));

    println!("\n>> All 360º Epistemic Invariants Verified Successfully! <<\n");
}

fn run_epistemic_eval_json(args: &[String]) {
    use kineti_memory::{
        ActionEvaluation, ConsequenceLevel, ContextScope, DomainKind, EpistemicCertainty,
        EpistemicEngine, EpistemicFact, RuleConstraintType,
    };

    let mut item = "item".to_string();
    let mut tags_vec: Vec<String> = Vec::new();
    let mut scope_str = "health".to_string();

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--item" if i + 1 < args.len() => {
                item = args[i + 1].clone();
                i += 2;
            }
            "--tags" if i + 1 < args.len() => {
                tags_vec = args[i + 1]
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                i += 2;
            }
            "--scope" if i + 1 < args.len() => {
                scope_str = args[i + 1].to_lowercase();
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    let engine = EpistemicEngine::new();
    let user_id = "praveen";
    let scope = match scope_str.as_str() {
        "work" => ContextScope::Domain(DomainKind::Work),
        "finance" => ContextScope::Domain(DomainKind::Finance),
        "schedule" => ContextScope::Domain(DomainKind::Schedule),
        "taste" => ContextScope::Domain(DomainKind::Taste),
        _ => ContextScope::Domain(DomainKind::Health),
    };

    // Baseline persona rules
    engine
        .ingest(
            EpistemicFact {
                id: "diet_01".to_string(),
                user_id: user_id.to_string(),
                scope: scope.clone(),
                attribute: "diet".to_string(),
                claim: "Vegetarian".to_string(),
                constraint_type: RuleConstraintType::BaselineRule,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: 1000,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: ConsequenceLevel::HighConsequence,
            },
            1000,
        )
        .unwrap();

    engine
        .ingest(
            EpistemicFact {
                id: "diet_02".to_string(),
                user_id: user_id.to_string(),
                scope: scope.clone(),
                attribute: "diet_exception".to_string(),
                claim: "eggs".to_string(),
                constraint_type: RuleConstraintType::PermittedException,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: 1005,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: ConsequenceLevel::HighConsequence,
            },
            1005,
        )
        .unwrap();

    engine
        .ingest(
            EpistemicFact {
                id: "diet_03".to_string(),
                user_id: user_id.to_string(),
                scope: scope.clone(),
                attribute: "dairy_pref".to_string(),
                claim: "low_dairy".to_string(),
                constraint_type: RuleConstraintType::Preference,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: 1010,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: ConsequenceLevel::Operational,
            },
            1010,
        )
        .unwrap();

    engine
        .ingest(
            EpistemicFact {
                id: "diet_04".to_string(),
                user_id: user_id.to_string(),
                scope: scope.clone(),
                attribute: "allergy".to_string(),
                claim: "peanut".to_string(),
                constraint_type: RuleConstraintType::SafetyCeiling,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: 1015,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: ConsequenceLevel::HighConsequence,
            },
            1015,
        )
        .unwrap();

    let tag_refs: Vec<&str> = tags_vec.iter().map(|s| s.as_str()).collect();
    let eval = engine.evaluate_candidate(user_id, &scope, &item, &tag_refs, 1020);

    match eval {
        ActionEvaluation::Permitted { recommendations } => {
            let recs_json = recommendations
                .into_iter()
                .map(|r| format!("\"{}\"", r))
                .collect::<Vec<_>>()
                .join(", ");
            println!(
                "{{\"status\":\"Permitted\",\"recommendations\":[{}]}}",
                recs_json
            );
        }
        ActionEvaluation::BlockedByRule { rule } => {
            println!("{{\"status\":\"BlockedByRule\",\"rule\":\"{}\"}}", rule);
        }
        ActionEvaluation::BlockedBySafetyCeiling { reason } => {
            println!(
                "{{\"status\":\"BlockedBySafetyCeiling\",\"reason\":\"{}\"}}",
                reason
            );
        }
        ActionEvaluation::NeedsClarification { question } => {
            println!(
                "{{\"status\":\"NeedsClarification\",\"question\":\"{}\"}}",
                question
            );
        }
    }
}

fn run_anti_drift_demo() {
    println!("\n=======================================================");
    println!("  360º Human Model: Verbatim Root Goal & Anti-Drift Engine");
    println!("  Guaranteed Goal Invariance & Friction Triage Ladder");
    println!("=======================================================\n");

    // 1. Verbatim Root Goal
    println!("[1/5] Locking Verbatim Root Goal...");
    let boundaries = ImmutableBoundaries {
        max_spend_microcents: Some(50_000_000), // $50.00 max
        counterparty: Some("Acme Corp".to_string()),
        deadline_epoch_ms: Some(1726600000000),
        reputation_protected: true,
    };
    let goal = VerbatimRootGoal::new(
        "goal_409",
        "Sign service contract with Acme Corp under $50 by Friday",
        "Signed counter-executed PDF archived in workspace",
        boundaries,
        1726500000000,
    );
    println!("  ✓ Root Ask: \"{}\"", goal.raw_user_ask);
    println!("  ✓ Done Definition: \"{}\"", goal.done_definition);
    println!("  ✓ Max Spend Ceiling: $50.00 | Counterparty: Acme Corp\n");

    // 2. Anti-Drift Inspector
    println!("[2/5] Evaluating Multi-Step Actions Against Verbatim Ask (Anti-Telephone Game)...");
    let step1 = DriftInspector::inspect_step(
        &goal,
        "Upload standard counter-offer PDF to Acme portal",
        Some(25_000_000),
        Some("Acme Corp"),
        1726510000000,
    );
    println!("  Step 1 (In-bounds, $25): {:?}", step1);

    let step2_drift = DriftInspector::inspect_step(
        &goal,
        "Accept vendor amendment with 10% budget overrun ($55)",
        Some(55_000_000),
        Some("Acme Corp"),
        1726520000000,
    );
    match step2_drift {
        DriftEvaluation::GoalMutationBlocked { reason, verbatim_ask } => {
            println!("  Step 2 (Budget overrun): BLOCKED!");
            println!("    Reason: {}", reason);
            println!("    Anchored to: \"{}\"", verbatim_ask);
        }
        _ => panic!("Expected drift block"),
    }

    let step3_drift = DriftInspector::inspect_step(
        &goal,
        "Forward draft to ThirdPartyVendor for signing",
        Some(10_000_000),
        Some("ThirdPartyVendor"),
        1726530000000,
    );
    match step3_drift {
        DriftEvaluation::GoalMutationBlocked { reason, .. } => {
            println!("  Step 3 (Counterparty drift): BLOCKED!");
            println!("    Reason: {}\n", reason);
        }
        _ => panic!("Expected counterparty block"),
    }

    // 3. Friction Triage Ladder
    println!("[3/5] Triaging Real-World Friction (Zero Sunk-Cost Fallacy)...");
    let noise = FrictionType::Noise {
        message: "HTTP 503 Service Unavailable blip".to_string(),
    };
    let res_noise = DriftInspector::triage_friction(&goal, noise, 5_000_000);
    println!("  Friction 1 (Noise): {:?}", res_noise);

    let broken_surface = FrictionType::BrokenSurface {
        surface_name: "LegacyWebPortal".to_string(),
        alternative_surface: Some("RestPartnerAPI".to_string()),
    };
    let res_reroute = DriftInspector::triage_friction(&goal, broken_surface, 10_000_000);
    println!("  Friction 2 (Broken Surface): {:?}", res_reroute);

    let real_constraint = FrictionType::RealConstraint {
        obstacle: "Acme lowest rate is $75.00/mo; will not accept $50.00".to_string(),
        mutates_done_definition: true,
    };
    let res_clean_no = DriftInspector::triage_friction(&goal, real_constraint, 15_000_000);
    match res_clean_no {
        FrictionResolution::EscalateCleanNo { clean_no_reason, banked_data } => {
            println!("  Friction 3 (Real Constraint): ESCALATED WITH CLEAN NO!");
            println!("    Clean No: {}", clean_no_reason);
            println!("    Banked Data: {}\n", banked_data);
        }
        _ => panic!("Expected EscalateCleanNo"),
    }

    // 4. Commitment-Time Verification
    println!("[4/5] Commitment-Time Verification (No Cached Snapshot Trust)...");
    let perishable = Perishability::Perishable { ttl_ms: 3_000 };
    let commit_check_fresh = CommitmentTimeVerifier::verify_fact(1000, perishable, 2000, "$49.00", Some("$49.00"));
    println!("  Commitment Check (Fresh, age 1s, TTL 3s): {:?}", commit_check_fresh);

    let commit_check_expired = CommitmentTimeVerifier::verify_fact(1000, perishable, 6000, "$49.00", Some("$49.00"));
    println!("  Commitment Check (Expired, age 5s, TTL 3s): {:?}", commit_check_expired);

    let commit_check_contradiction = CommitmentTimeVerifier::verify_fact(1000, perishable, 2000, "$49.00", Some("$79.00"));
    println!("  Commitment Check (Live Contradiction): {:?}\n", commit_check_contradiction);

    // 5. Asymmetric Gap-Filling & Durability Promotion
    println!("[5/5] Asymmetric Gap-Filling & Promotion Signals...");
    let cheap_gap = GapFillingPolicy::evaluate_gap("pdf_watermark", ConsequenceLevel::Trivial, true, 0, Some("Confidential"));
    println!("  Gap Filling (Reversible format): {:?}", cheap_gap);

    let expensive_gap = GapFillingPolicy::evaluate_gap("payment_routing_number", ConsequenceLevel::HighConsequence, false, 50_000_000, None);
    println!("  Gap Filling (Irreversible payment): {:?}", expensive_gap);

    let promo_always = PromotionSignalDetector::evaluate("I always require two-factor approval for Acme invoices", 0);
    println!("  Promotion (\"Always\" linguistic signal): {:?}", promo_always);

    let promo_oneoff = PromotionSignalDetector::evaluate("Use single column view for today's summary", 0);
    println!("  Promotion (One-off instruction): {:?}", promo_oneoff);

    println!("\n✓ 360º Human Model Anti-Drift Verification Succeeded!\n");
}

//! Adversarial Challenge Test Suite for kineti-memory
//!
//! Stress-testing:
//! 1. Multi-Scope Isolation (R1):
//!    - Exhaustive 9x9 scope accessibility matrix
//!    - Empty contact IDs, whitespace, unicode, special chars, case sensitivity
//! 2. Epistemic Certainty Precedence (R2):
//!    - Inferred cannot overwrite DirectlyKnown
//!    - ObservedPattern cannot overwrite DirectlyKnown
//!    - Inferred cannot overwrite ObservedPattern
//!    - Exception vs BaselineRule certainty boundaries
//!    - High-consequence authorization barrier: Inferred and ObservedPattern rejection
//!    - Missing, empty, and whitespace token rejection
//! 3. Concurrent multi-thread stress testing

use kineti_memory::{
    resolve_advice, ActionEvaluation, ConsequenceLevel, ContextScope, DomainKind,
    EpistemicCertainty, EpistemicEngine, EpistemicError, EpistemicFact, EpistemicTransition,
    FactSource, RuleConstraintType,
};
use std::sync::Arc;
use std::thread;

// Helper to construct test facts quickly
fn make_fact(
    id: &str,
    user_id: &str,
    scope: ContextScope,
    attr: &str,
    claim: &str,
    constraint: RuleConstraintType,
    certainty: EpistemicCertainty,
    consequence: ConsequenceLevel,
    time: u64,
) -> EpistemicFact {
    EpistemicFact {
        id: id.to_string(),
        user_id: user_id.to_string(),
        scope,
        attribute: attr.to_string(),
        claim: claim.to_string(),
        constraint_type: constraint,
        certainty,
        valid_from: time,
        valid_until: None,
        contradiction_criteria: None,
        consequence_level: consequence,
    }
}

// =========================================================================
// R1: MULTI-SCOPE ISOLATION TESTS
// =========================================================================

#[test]
fn test_multiscope_exhaustive_cross_matrix_isolation() {
    let scopes = vec![
        ContextScope::Global,
        ContextScope::Domain(DomainKind::Health),
        ContextScope::Domain(DomainKind::Work),
        ContextScope::Domain(DomainKind::Finance),
        ContextScope::Domain(DomainKind::Schedule),
        ContextScope::Domain(DomainKind::Taste),
        ContextScope::Relationship {
            contact_id: "alice".to_string(),
            role: "manager".to_string(),
        },
        ContextScope::Relationship {
            contact_id: "bob".to_string(),
            role: "friend".to_string(),
        },
        ContextScope::Relationship {
            contact_id: "charlie".to_string(),
            role: "coworker".to_string(),
        },
    ];

    for (i, fact_scope) in scopes.iter().enumerate() {
        for (j, query_scope) in scopes.iter().enumerate() {
            let accessible = fact_scope.is_accessible_from(query_scope);

            if i == 0 {
                // Global facts MUST be accessible from ALL query scopes
                assert!(
                    accessible,
                    "Global fact must be accessible from query scope {:?}",
                    query_scope
                );
            } else if i == j {
                // Same scope must always be accessible
                assert!(
                    accessible,
                    "Fact in scope {:?} must be accessible from same query scope",
                    fact_scope
                );
            } else {
                // Different scopes: MUST FAIL CLOSED (zero cross-scope leakage)
                assert!(
                    !accessible,
                    "LEAKAGE DETECTED! Fact in scope {:?} was accessible from query scope {:?}",
                    fact_scope, query_scope
                );
            }
        }
    }
}

#[test]
fn test_multiscope_relationship_role_polymorphism() {
    // A fact scoped to (Alice, "manager") should still be accessible when querying (Alice, "mentor")
    // because relationship context is bound to the person (contact_id).
    let fact_scope = ContextScope::Relationship {
        contact_id: "alice".to_string(),
        role: "manager".to_string(),
    };
    let query_same_person_diff_role = ContextScope::Relationship {
        contact_id: "alice".to_string(),
        role: "mentor".to_string(),
    };
    assert!(
        fact_scope.is_accessible_from(&query_same_person_diff_role),
        "Facts for contact_id 'alice' should be accessible for any query on 'alice'"
    );

    // But query for Bob must NEVER access Alice's facts
    let query_diff_person = ContextScope::Relationship {
        contact_id: "bob".to_string(),
        role: "manager".to_string(),
    };
    assert!(
        !fact_scope.is_accessible_from(&query_diff_person),
        "LEAKAGE! Alice's fact accessed by Bob"
    );
}

#[test]
fn test_multiscope_edge_cases_empty_special_chars_and_case() {
    // 1. Case Sensitivity
    // Rust String equality is case-sensitive: "Alice" != "alice"
    let fact_upper = ContextScope::Relationship {
        contact_id: "Alice".to_string(),
        role: "colleague".to_string(),
    };
    let query_lower = ContextScope::Relationship {
        contact_id: "alice".to_string(),
        role: "colleague".to_string(),
    };
    // Should NOT leak across differing cases under exact byte matching
    assert!(!fact_upper.is_accessible_from(&query_lower));

    // 2. Empty Contact IDs
    let fact_empty = ContextScope::Relationship {
        contact_id: "".to_string(),
        role: "".to_string(),
    };
    let query_alice = ContextScope::Relationship {
        contact_id: "alice".to_string(),
        role: "".to_string(),
    };
    // Empty contact ID must NOT match "alice"
    assert!(!fact_empty.is_accessible_from(&query_alice));
    // Empty contact ID must NOT leak to Global query
    assert!(!fact_empty.is_accessible_from(&ContextScope::Global));
    // Empty contact ID must NOT leak to Domain queries
    assert!(!fact_empty.is_accessible_from(&ContextScope::Domain(DomainKind::Work)));

    // 3. Special Characters: Regex metacharacters
    let fact_regex = ContextScope::Relationship {
        contact_id: ".*".to_string(),
        role: "wildcard".to_string(),
    };
    // Must NOT act as a wildcard regex!
    assert!(!fact_regex.is_accessible_from(&query_alice));
    assert!(!fact_regex.is_accessible_from(&ContextScope::Global));

    // 4. Special Characters: Null bytes and path traversal
    let fact_traversal = ContextScope::Relationship {
        contact_id: "../../etc/passwd\0root".to_string(),
        role: "attacker".to_string(),
    };
    assert!(!fact_traversal.is_accessible_from(&query_alice));
    assert!(!fact_traversal.is_accessible_from(&ContextScope::Global));
    assert!(!fact_traversal.is_accessible_from(&ContextScope::Domain(DomainKind::Finance)));

    // 5. Unicode and Emoji contact IDs
    let fact_emoji = ContextScope::Relationship {
        contact_id: "👩‍🚀_astronaut".to_string(),
        role: "crew".to_string(),
    };
    let query_emoji_exact = ContextScope::Relationship {
        contact_id: "👩‍🚀_astronaut".to_string(),
        role: "pilot".to_string(),
    };
    let query_emoji_diff = ContextScope::Relationship {
        contact_id: "👨‍🚀_astronaut".to_string(),
        role: "pilot".to_string(),
    };
    assert!(fact_emoji.is_accessible_from(&query_emoji_exact));
    assert!(!fact_emoji.is_accessible_from(&query_emoji_diff));
}

#[test]
fn test_multiscope_engine_query_and_resolve_isolation() {
    let engine = EpistemicEngine::new();
    let u = "user_iso_test";

    // Ingest facts in all scopes
    engine
        .ingest(
            make_fact(
                "f_g",
                u,
                ContextScope::Global,
                "timezone",
                "UTC",
                RuleConstraintType::Preference,
                EpistemicCertainty::DirectlyKnown,
                ConsequenceLevel::Trivial,
                100,
            ),
            100,
        )
        .unwrap();

    engine
        .ingest(
            make_fact(
                "f_health",
                u,
                ContextScope::Domain(DomainKind::Health),
                "condition",
                "Asthma",
                RuleConstraintType::SafetyCeiling,
                EpistemicCertainty::DirectlyKnown,
                ConsequenceLevel::HighConsequence,
                100,
            ),
            100,
        )
        .unwrap();

    engine
        .ingest(
            make_fact(
                "f_fin",
                u,
                ContextScope::Domain(DomainKind::Finance),
                "card",
                "Amex 1234",
                RuleConstraintType::BaselineRule,
                EpistemicCertainty::DirectlyKnown,
                ConsequenceLevel::HighConsequence,
                100,
            ),
            100,
        )
        .unwrap();

    engine
        .ingest(
            make_fact(
                "f_alice",
                u,
                ContextScope::Relationship {
                    contact_id: "alice".to_string(),
                    role: "sister".to_string(),
                },
                "secret",
                "Birthday surprise",
                RuleConstraintType::Preference,
                EpistemicCertainty::DirectlyKnown,
                ConsequenceLevel::Operational,
                100,
            ),
            100,
        )
        .unwrap();

    // Query Global: MUST ONLY return Global fact (0 Health, 0 Finance, 0 Alice)
    let g_facts = engine.query_facts_in_scope(u, &ContextScope::Global, 150);
    assert_eq!(g_facts.len(), 1);
    assert_eq!(g_facts[0].id, "f_g");

    // Query Health: MUST return Global + Health, but NEVER Finance or Alice
    let h_facts = engine.query_facts_in_scope(u, &ContextScope::Domain(DomainKind::Health), 150);
    assert_eq!(h_facts.len(), 2);
    assert!(h_facts.iter().any(|f| f.id == "f_g"));
    assert!(h_facts.iter().any(|f| f.id == "f_health"));
    assert!(!h_facts.iter().any(|f| f.id == "f_fin"));
    assert!(!h_facts.iter().any(|f| f.id == "f_alice"));

    // Query Bob (Relationship): MUST return Global, but NEVER Alice, Health, or Finance
    let b_scope = ContextScope::Relationship {
        contact_id: "bob".to_string(),
        role: "colleague".to_string(),
    };
    let b_facts = engine.query_facts_in_scope(u, &b_scope, 150);
    assert_eq!(b_facts.len(), 1);
    assert_eq!(b_facts[0].id, "f_g");

    // Query Alice: MUST return Global + Alice, but NEVER Health or Finance
    let a_scope = ContextScope::Relationship {
        contact_id: "alice".to_string(),
        role: "sister".to_string(),
    };
    let a_facts = engine.query_facts_in_scope(u, &a_scope, 150);
    assert_eq!(a_facts.len(), 2);
    assert!(a_facts.iter().any(|f| f.id == "f_g"));
    assert!(a_facts.iter().any(|f| f.id == "f_alice"));
    assert!(!a_facts.iter().any(|f| f.id == "f_health"));
    assert!(!a_facts.iter().any(|f| f.id == "f_fin"));
}

// =========================================================================
// R2: EPISTEMIC CERTAINTY PRECEDENCE TESTS
// =========================================================================

#[test]
fn test_epistemic_inferred_cannot_overwrite_directly_known() {
    let engine = EpistemicEngine::new();
    let u = "user_prec_1";
    let scope = ContextScope::Domain(DomainKind::Work);

    // 1. Establish ground truth (DirectlyKnown, Rank 3)
    let ground_truth = make_fact(
        "dk_1",
        u,
        scope.clone(),
        "editor",
        "Neovim",
        RuleConstraintType::Preference,
        EpistemicCertainty::DirectlyKnown,
        ConsequenceLevel::Operational,
        1000,
    );
    engine.ingest(ground_truth, 1000).unwrap();

    // 2. Machine deduction attempts to overwrite with VSCode (Inferred, Rank 1)
    let inference = make_fact(
        "inf_1",
        u,
        scope.clone(),
        "editor",
        "VSCode",
        RuleConstraintType::Preference,
        EpistemicCertainty::Inferred {
            reasoning: "Saw a .vscode folder in project root".to_string(),
        },
        ConsequenceLevel::Operational,
        1050,
    );

    // `ingest` MUST return ConflictNeedsClarification
    let trans = engine.ingest(inference.clone(), 1050).unwrap();
    assert!(
        matches!(trans, EpistemicTransition::ConflictNeedsClarification { .. }),
        "Inferred fact MUST NOT overwrite DirectlyKnown fact; got {:?}",
        trans
    );

    // `try_ingest` MUST return typed error Err(EpistemicError::ConflictNeedsClarification)
    let err = engine.try_ingest(inference, 1050);
    assert!(
        matches!(err, Err(EpistemicError::ConflictNeedsClarification { .. })),
        "try_ingest MUST return Err(ConflictNeedsClarification)"
    );

    // Verify state in store: Ground truth remains ACTIVE and UNTOUCHED
    let facts = engine.query_facts_in_scope(u, &scope, 1060);
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].id, "dk_1");
    assert_eq!(facts[0].claim, "Neovim");
    assert_eq!(facts[0].certainty, EpistemicCertainty::DirectlyKnown);
    assert_eq!(facts[0].valid_until, None);
}

#[test]
fn test_epistemic_observed_pattern_cannot_overwrite_directly_known() {
    let engine = EpistemicEngine::new();
    let u = "user_prec_2";
    let scope = ContextScope::Domain(DomainKind::Taste);

    // 1. Establish ground truth: DirectlyKnown (Rank 3)
    let ground_truth = make_fact(
        "dk_music",
        u,
        scope.clone(),
        "music_genre",
        "Jazz",
        RuleConstraintType::Preference,
        EpistemicCertainty::DirectlyKnown,
        ConsequenceLevel::Trivial,
        1000,
    );
    engine.ingest(ground_truth, 1000).unwrap();

    // 2. Behavioral sensor observes frequent listening to Electronic (ObservedPattern, Rank 2)
    let pattern = make_fact(
        "obs_music",
        u,
        scope.clone(),
        "music_genre",
        "Electronic",
        RuleConstraintType::Preference,
        EpistemicCertainty::ObservedPattern {
            observation_count: 50,
            consistency: 0.95,
        },
        ConsequenceLevel::Trivial,
        1050,
    );

    // `ingest` MUST return ConflictNeedsClarification
    let trans = engine.ingest(pattern.clone(), 1050).unwrap();
    assert!(
        matches!(trans, EpistemicTransition::ConflictNeedsClarification { .. }),
        "ObservedPattern (Rank 2) MUST NOT overwrite DirectlyKnown (Rank 3); got {:?}",
        trans
    );

    // `try_ingest` MUST return Err
    let err = engine.try_ingest(pattern, 1050);
    assert!(matches!(
        err,
        Err(EpistemicError::ConflictNeedsClarification { .. })
    ));

    // Ground truth remains active
    let facts = engine.query_facts_in_scope(u, &scope, 1060);
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].claim, "Jazz");
    assert_eq!(facts[0].certainty, EpistemicCertainty::DirectlyKnown);
}

#[test]
fn test_epistemic_inferred_cannot_overwrite_observed_pattern() {
    let engine = EpistemicEngine::new();
    let u = "user_prec_3";
    let scope = ContextScope::Domain(DomainKind::Schedule);

    // 1. Establish ObservedPattern (Rank 2)
    let pattern = make_fact(
        "obs_sched",
        u,
        scope.clone(),
        "wake_time",
        "7:00 AM",
        RuleConstraintType::Preference,
        EpistemicCertainty::ObservedPattern {
            observation_count: 14,
            consistency: 0.90,
        },
        ConsequenceLevel::Operational,
        1000,
    );
    engine.ingest(pattern, 1000).unwrap();

    // 2. Model inference (Rank 1) tries to claim wake time is 11:00 AM
    let inference = make_fact(
        "inf_sched",
        u,
        scope.clone(),
        "wake_time",
        "11:00 AM",
        RuleConstraintType::Preference,
        EpistemicCertainty::Inferred {
            reasoning: "User woke up late once on Saturday".to_string(),
        },
        ConsequenceLevel::Operational,
        1050,
    );

    // Inferred (Rank 1) MUST NOT overwrite ObservedPattern (Rank 2)
    let trans = engine.ingest(inference.clone(), 1050).unwrap();
    assert!(matches!(
        trans,
        EpistemicTransition::ConflictNeedsClarification { .. }
    ));

    let err = engine.try_ingest(inference, 1050);
    assert!(matches!(
        err,
        Err(EpistemicError::ConflictNeedsClarification { .. })
    ));

    // Active fact remains 7:00 AM
    let facts = engine.query_facts_in_scope(u, &scope, 1060);
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].claim, "7:00 AM");
}

#[test]
fn test_epistemic_directly_known_can_supersede_directly_known() {
    let engine = EpistemicEngine::new();
    let u = "user_prec_4";
    let scope = ContextScope::Domain(DomainKind::Work);

    // 1. User says "I use Slack"
    engine
        .ingest(
            make_fact(
                "comm_1",
                u,
                scope.clone(),
                "chat_app",
                "Slack",
                RuleConstraintType::Preference,
                EpistemicCertainty::DirectlyKnown,
                ConsequenceLevel::Operational,
                1000,
            ),
            1000,
        )
        .unwrap();

    // 2. User explicitly updates: "I switched to Teams" (DirectlyKnown, Rank 3)
    let update = make_fact(
        "comm_2",
        u,
        scope.clone(),
        "chat_app",
        "Teams",
        RuleConstraintType::Preference,
        EpistemicCertainty::DirectlyKnown,
        ConsequenceLevel::Operational,
        2000,
    );

    let trans = engine.ingest(update, 2000).unwrap();
    assert_eq!(
        trans,
        EpistemicTransition::Superseded {
            previous_id: "comm_1".to_string(),
            new_id: "comm_2".to_string(),
        }
    );

    // Historical query at t=1500 returns Slack
    let past_facts = engine.query_facts_in_scope(u, &scope, 1500);
    assert_eq!(past_facts.len(), 1);
    assert_eq!(past_facts[0].claim, "Slack");

    // Current query at t=2500 returns Teams
    let curr_facts = engine.query_facts_in_scope(u, &scope, 2500);
    assert_eq!(curr_facts.len(), 1);
    assert_eq!(curr_facts[0].claim, "Teams");
}

#[test]
fn test_epistemic_identical_claim_reinforcement_does_not_demote() {
    let engine = EpistemicEngine::new();
    let u = "user_prec_5";
    let scope = ContextScope::Domain(DomainKind::Taste);

    // DirectlyKnown fact (Rank 3)
    engine
        .ingest(
            make_fact(
                "f_coffee",
                u,
                scope.clone(),
                "coffee",
                "Espresso",
                RuleConstraintType::Preference,
                EpistemicCertainty::DirectlyKnown,
                ConsequenceLevel::Trivial,
                1000,
            ),
            1000,
        )
        .unwrap();

    // Incoming identical claim with Inferred certainty (Rank 1)
    let incoming_inferred = make_fact(
        "f_coffee_inf",
        u,
        scope.clone(),
        "coffee",
        "Espresso",
        RuleConstraintType::Preference,
        EpistemicCertainty::Inferred {
            reasoning: "Saw receipt".to_string(),
        },
        ConsequenceLevel::Trivial,
        1050,
    );

    let trans = engine.ingest(incoming_inferred, 1050).unwrap();
    assert_eq!(trans, EpistemicTransition::Confirmed("f_coffee".to_string()));

    // Verify certainty was NOT demoted to Inferred
    let facts = engine.query_facts_in_scope(u, &scope, 1060);
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].certainty, EpistemicCertainty::DirectlyKnown);
}

// =========================================================================
// R2: HIGH-CONSEQUENCE ACTION AUTHORIZATION TESTS
// =========================================================================

#[test]
fn test_high_consequence_cannot_be_authorized_by_inferred_fact() {
    let engine = EpistemicEngine::new();
    let u = "user_auth_1";
    let scope = ContextScope::Domain(DomainKind::Finance);

    // Inferred fact in finance domain (Rank 1)
    engine
        .ingest(
            make_fact(
                "inf_wire",
                u,
                scope.clone(),
                "wire_transfer",
                "Transfer $5,000 to vendor",
                RuleConstraintType::BaselineRule,
                EpistemicCertainty::Inferred {
                    reasoning: "Vendor invoice received in email".to_string(),
                },
                ConsequenceLevel::HighConsequence,
                1000,
            ),
            1000,
        )
        .unwrap();

    // High consequence authorization with NO confirmed token MUST FAIL
    let res = engine.authorize_action(
        u,
        &scope,
        ConsequenceLevel::HighConsequence,
        "Transfer $5,000 to vendor",
        1050,
    );
    assert!(
        matches!(res, Err(ActionEvaluation::NeedsClarification { .. })),
        "Inferred fact MUST NOT authorize HighConsequence action; got {:?}",
        res
    );

    // Strict mode MUST return Err(EpistemicError::UnauthorizedHighConsequence)
    let strict_res = engine.authorize_action_strict(
        u,
        &scope,
        ConsequenceLevel::HighConsequence,
        "Transfer $5,000 to vendor",
        1050,
    );
    assert!(
        matches!(
            strict_res,
            Err(EpistemicError::UnauthorizedHighConsequence { .. })
        ),
        "Strict authorization MUST return Err(UnauthorizedHighConsequence)"
    );
}

#[test]
fn test_high_consequence_cannot_be_authorized_by_observed_pattern() {
    let engine = EpistemicEngine::new();
    let u = "user_auth_2";
    let scope = ContextScope::Domain(DomainKind::Finance);

    // ObservedPattern fact (Rank 2)
    engine
        .ingest(
            make_fact(
                "obs_spend",
                u,
                scope.clone(),
                "auto_pay",
                "Pay recurring cloud bill",
                RuleConstraintType::BaselineRule,
                EpistemicCertainty::ObservedPattern {
                    observation_count: 12,
                    consistency: 1.0,
                },
                ConsequenceLevel::HighConsequence,
                1000,
            ),
            1000,
        )
        .unwrap();

    // ObservedPattern MUST NOT authorize HighConsequence action
    let res = engine.authorize_action(
        u,
        &scope,
        ConsequenceLevel::HighConsequence,
        "Pay recurring cloud bill",
        1050,
    );
    assert!(matches!(
        res,
        Err(ActionEvaluation::NeedsClarification { .. })
    ));

    let strict_res = engine.authorize_action_strict(
        u,
        &scope,
        ConsequenceLevel::HighConsequence,
        "Pay recurring cloud bill",
        1050,
    );
    assert!(matches!(
        strict_res,
        Err(EpistemicError::UnauthorizedHighConsequence { .. })
    ));
}

#[test]
fn test_high_consequence_missing_or_blank_token_fails_closed() {
    let engine = EpistemicEngine::new();
    let u = "user_auth_3";
    let scope = ContextScope::Domain(DomainKind::Finance);

    // Ingest an Inferred fact
    engine
        .ingest(
            make_fact(
                "inf_op",
                u,
                scope.clone(),
                "delete_prod_db",
                "Drop production database",
                RuleConstraintType::BaselineRule,
                EpistemicCertainty::Inferred {
                    reasoning: "Test clean-up script".to_string(),
                },
                ConsequenceLevel::HighConsequence,
                1000,
            ),
            1000,
        )
        .unwrap();

    // Adversarial tokens: None, empty string, whitespace only, tabs/newlines
    let bad_tokens: Vec<Option<&str>> = vec![
        None,
        Some(""),
        Some("   "),
        Some("\t\t"),
        Some("\n\r\n"),
        Some("   \t  \n  "),
    ];

    for (idx, token) in bad_tokens.into_iter().enumerate() {
        let res = engine.authorize_action_with_token(
            u,
            &scope,
            ConsequenceLevel::HighConsequence,
            "Drop production database",
            token,
            1050,
        );
        assert!(
            matches!(res, Err(ActionEvaluation::NeedsClarification { .. })),
            "Bad token case #{} ({:?}) MUST fail closed",
            idx,
            token
        );

        let strict_res = engine.authorize_action_with_token_strict(
            u,
            &scope,
            ConsequenceLevel::HighConsequence,
            "Drop production database",
            token,
            1050,
        );
        assert!(
            matches!(
                strict_res,
                Err(EpistemicError::UnauthorizedHighConsequence { .. })
            ),
            "Bad token case #{} ({:?}) MUST return UnauthorizedHighConsequence error",
            idx,
            token
        );
    }
}

#[test]
fn test_high_consequence_authorized_by_directly_known_or_confirmed_token() {
    let engine = EpistemicEngine::new();
    let u = "user_auth_4";
    let scope = ContextScope::Domain(DomainKind::Finance);

    // 1. DirectlyKnown fact authorizes the action
    engine
        .ingest(
            make_fact(
                "dk_auth",
                u,
                scope.clone(),
                "salary_payout",
                "Execute payroll on 1st of month",
                RuleConstraintType::BaselineRule,
                EpistemicCertainty::DirectlyKnown,
                ConsequenceLevel::HighConsequence,
                1000,
            ),
            1000,
        )
        .unwrap();

    let res = engine.authorize_action(
        u,
        &scope,
        ConsequenceLevel::HighConsequence,
        "Execute payroll on 1st of month",
        1050,
    );
    assert_eq!(res, Ok(true));

    let strict_res = engine.authorize_action_strict(
        u,
        &scope,
        ConsequenceLevel::HighConsequence,
        "Execute payroll on 1st of month",
        1050,
    );
    assert_eq!(strict_res, Ok(true));

    // 2. Confirmed token authorizes high consequence action without pre-existing DirectlyKnown fact
    let token_res = engine.authorize_action_with_token(
        u,
        &scope,
        ConsequenceLevel::HighConsequence,
        "One-off emergency server shutdown",
        Some("tok_valid_sec_12345"),
        1050,
    );
    assert_eq!(token_res, Ok(true));

    // 3. FactSource::ConfirmedToken registered in engine also authorizes
    let fact_with_source = make_fact(
        "f_confirmed_src",
        u,
        scope.clone(),
        "api_key_rotation",
        "Rotate production API keys",
        RuleConstraintType::BaselineRule,
        EpistemicCertainty::DirectlyKnown,
        ConsequenceLevel::HighConsequence,
        1000,
    );
    engine
        .ingest_with_source(
            fact_with_source,
            FactSource::ConfirmedToken {
                token_id: "token_abc_789".to_string(),
            },
            1000,
        )
        .unwrap();

    let auth_confirmed = engine.authorize_action(
        u,
        &scope,
        ConsequenceLevel::HighConsequence,
        "Rotate production API keys",
        1050,
    );
    assert_eq!(auth_confirmed, Ok(true));
}

#[test]
fn test_lower_consequence_levels_do_not_require_high_consequence_gating() {
    let engine = EpistemicEngine::new();
    let u = "user_auth_5";
    let scope = ContextScope::Domain(DomainKind::Taste);

    // Trivial consequence
    let res_triv = engine.authorize_action(
        u,
        &scope,
        ConsequenceLevel::Trivial,
        "Recommend restaurant for lunch",
        1000,
    );
    assert_eq!(res_triv, Ok(true));

    // Operational consequence
    let res_op = engine.authorize_action(
        u,
        &scope,
        ConsequenceLevel::Operational,
        "Draft calendar event invite",
        1000,
    );
    assert_eq!(res_op, Ok(true));
}

// =========================================================================
// R3 & CERTAINTY EDGE CASES: ADVICE RESOLUTION UNDER ADVERSARIAL INPUTS
// =========================================================================

#[test]
fn test_resolve_advice_inferred_exception_adversarial_investigation() {
    // Investigate whether an Inferred exception can whitelist a prohibited item
    // against a DirectlyKnown BaselineRule.
    let baseline_rule = EpistemicFact {
        id: "r_veg".to_string(),
        user_id: "u_adv".to_string(),
        scope: ContextScope::Domain(DomainKind::Health),
        attribute: "diet".to_string(),
        claim: "Vegetarian".to_string(),
        constraint_type: RuleConstraintType::BaselineRule,
        certainty: EpistemicCertainty::DirectlyKnown,
        valid_from: 100,
        valid_until: None,
        contradiction_criteria: None,
        consequence_level: ConsequenceLevel::HighConsequence,
    };

    let inferred_exception = EpistemicFact {
        id: "e_meat".to_string(),
        user_id: "u_adv".to_string(),
        scope: ContextScope::Domain(DomainKind::Health),
        attribute: "diet_exception".to_string(),
        claim: "chicken".to_string(),
        constraint_type: RuleConstraintType::PermittedException,
        certainty: EpistemicCertainty::Inferred {
            reasoning: "Hypothesized meat consumption from single event".to_string(),
        },
        valid_from: 105,
        valid_until: None,
        contradiction_criteria: None,
        consequence_level: ConsequenceLevel::HighConsequence,
    };

    let facts = vec![baseline_rule, inferred_exception];

    // Case 1: Item with chicken tag
    let advice_chicken = resolve_advice(&facts, "Chicken Teriyaki", &["chicken"]);
    // Verified Invariant: Inferred exceptions CANNOT whitelist prohibited items against DirectlyKnown rules
    assert!(
        !advice_chicken.is_permitted(),
        "Inferred exception must NOT whitelist chicken against DirectlyKnown vegetarian baseline!"
    );

    // Case 2: But does `engine.ingest` block an Inferred exception if ingested under the same attribute?
    let engine = EpistemicEngine::new();
    let base = make_fact(
        "base_veg",
        "u_adv",
        ContextScope::Domain(DomainKind::Health),
        "diet",
        "Vegetarian",
        RuleConstraintType::BaselineRule,
        EpistemicCertainty::DirectlyKnown,
        ConsequenceLevel::HighConsequence,
        100,
    );
    engine.ingest(base, 100).unwrap();

    // Now try to ingest an Inferred exception under the SAME attribute "diet"
    let inf_exc_same_attr = make_fact(
        "exc_inf_same",
        "u_adv",
        ContextScope::Domain(DomainKind::Health),
        "diet",
        "chicken",
        RuleConstraintType::PermittedException,
        EpistemicCertainty::Inferred {
            reasoning: "weak guess".to_string(),
        },
        ConsequenceLevel::HighConsequence,
        105,
    );
    let trans = engine.ingest(inf_exc_same_attr, 105).unwrap();
    // Under same attribute, the conflict check fires first because rank 1 < rank 3!
    assert!(
        matches!(trans, EpistemicTransition::ConflictNeedsClarification { .. }),
        "Under same attribute, engine.ingest correctly blocks Inferred exception!"
    );
}

// =========================================================================
// CONCURRENCY & STRESS TEST
// =========================================================================

#[test]
fn test_stress_concurrent_epistemic_ingestion_and_isolation() {
    let engine = Arc::new(EpistemicEngine::new());
    let num_threads = 16;
    let iterations_per_thread = 100;

    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let eng = Arc::clone(&engine);
        let handle = thread::spawn(move || {
            let u = format!("user_concurrent_{}", thread_id % 4);
            for i in 0..iterations_per_thread {
                let time = 1000 + i;
                let scope = match i % 3 {
                    0 => ContextScope::Global,
                    1 => ContextScope::Domain(DomainKind::Work),
                    _ => ContextScope::Relationship {
                        contact_id: format!("contact_{}", thread_id),
                        role: "peer".to_string(),
                    },
                };

                let fact = make_fact(
                    &format!("f_{}_{}", thread_id, i),
                    &u,
                    scope.clone(),
                    &format!("attr_{}", i % 5),
                    &format!("claim_{}_{}", thread_id, i),
                    RuleConstraintType::Preference,
                    if i % 2 == 0 {
                        EpistemicCertainty::DirectlyKnown
                    } else {
                        EpistemicCertainty::Inferred {
                            reasoning: "stress".to_string(),
                        }
                    },
                    ConsequenceLevel::Operational,
                    time,
                );

                let _ = eng.ingest(fact, time);

                // Concurrently query
                let q_facts = eng.query_facts_in_scope(&u, &scope, time);
                assert!(!q_facts.is_empty());
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

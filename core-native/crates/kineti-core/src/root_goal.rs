//! # Verbatim Root Goal & Anti-Drift Engine (`root_goal`)
//!
//! Implements the core invariants of the 360º Human Model:
//! - **Verbatim Ask Invariance**: The root goal is the user's exact words plus the definition of done.
//! - **Scaffolding is Expendable**: Plans, intermediate steps, and surfaces can be discarded without mutating the goal.
//! - **Anti-Drift Inspection**: Every step is validated against the original ask, preventing the multi-step "telephone game".
//! - **Friction Triage Ladder**: Distinguishes Noise (retry), Broken Surfaces (reroute), and Real Constraints (escalate with "Clean No").
//! - **Zero Sunk-Cost Fallacy**: When a branch dies, progress is banked as data and debris is written off.

use crate::Microcents;

/// Immutable boundaries of the root goal that can never be compromised by agent improvisation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImmutableBoundaries {
    /// Maximum financial spend in microcents ($1 = 1,000,000 microcents).
    pub max_spend_microcents: Option<Microcents>,
    /// Authorized counterparty or organization (e.g. "Vendor Corp", "Alok").
    pub counterparty: Option<String>,
    /// Hard deadline timestamp in milliseconds since epoch.
    pub deadline_epoch_ms: Option<u64>,
    /// Requirement for user reputation and public representation.
    pub reputation_protected: bool,
}

impl Default for ImmutableBoundaries {
    fn default() -> Self {
        Self {
            max_spend_microcents: None,
            counterparty: None,
            deadline_epoch_ms: None,
            reputation_protected: true,
        }
    }
}

/// The Verbatim Root Goal anchoring autonomous task execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerbatimRootGoal {
    /// Unique identifier for this goal instance.
    pub goal_id: String,
    /// Exact, unmodified text uttered by the user.
    pub raw_user_ask: String,
    /// Unambiguous specification of what outcome constitutes "Done".
    pub done_definition: String,
    /// Non-negotiable boundaries (money, counterparty, deadline, reputation).
    pub boundaries: ImmutableBoundaries,
    /// Timestamp when this goal was locked.
    pub locked_at: u64,
}

impl VerbatimRootGoal {
    /// Locks a new Verbatim Root Goal from exact user instructions.
    pub fn new(
        goal_id: impl Into<String>,
        raw_user_ask: impl Into<String>,
        done_definition: impl Into<String>,
        boundaries: ImmutableBoundaries,
        locked_at: u64,
    ) -> Self {
        Self {
            goal_id: goal_id.into(),
            raw_user_ask: raw_user_ask.into(),
            done_definition: done_definition.into(),
            boundaries,
            locked_at,
        }
    }
}

/// Type of friction encountered when interacting with third-party systems.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrictionType {
    /// Transient network blip, 503, or rate limit.
    Noise {
        /// Diagnostic message.
        message: String,
    },
    /// A specific API, portal, or website surface is broken or changed, but alternatives exist.
    BrokenSurface {
        /// The failed surface identifier.
        surface_name: String,
        /// Alternative surface available.
        alternative_surface: Option<String>,
    },
    /// A hard constraint from the third party (e.g. price ceiling exceeded, deadline refused).
    RealConstraint {
        /// Reason given by the third party.
        obstacle: String,
        /// Whether accommodating this would mutate what "Done" means.
        mutates_done_definition: bool,
    },
}

/// Decision made by the Friction Triage Ladder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrictionResolution {
    /// Level 1: Transient noise -> retry with exponential backoff.
    RetryWithBackoff {
        /// Recommended delay in milliseconds.
        delay_ms: u64,
    },
    /// Level 2: Broken surface -> reroute to alternative without bothering the user.
    SilentReroute {
        /// Alternative surface to engage.
        target_surface: String,
        /// Preserved verbatim goal.
        verbatim_goal: String,
    },
    /// Level 3: Real constraint -> escalate clean impossibility rather than quiet goal mutation.
    EscalateCleanNo {
        /// Exact reason why the task cannot be achieved inside the user's terms.
        clean_no_reason: String,
        /// Banked information from failed attempt.
        banked_data: String,
    },
}

/// Result of evaluating a candidate execution step for goal drift.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriftEvaluation {
    /// Step strictly serves the verbatim root goal within all immutable boundaries.
    InBounds,
    /// Step changes sequence or tool (flexible scaffolding), but done definition remains unchanged.
    PathReroutePermitted {
        /// Rationale for the reroute.
        rationale: String,
    },
    /// Step violates immutable boundaries or attempts to rewrite the definition of done.
    GoalMutationBlocked {
        /// Detailed reason for the block.
        reason: String,
        /// The original user ask that was violated.
        verbatim_ask: String,
    },
}

/// Anti-Drift Inspector validating candidate actions directly against the verbatim root ask.
#[derive(Debug)]
pub struct DriftInspector;

impl DriftInspector {
    /// Validates a candidate step against the verbatim root goal.
    ///
    /// Evaluates directly against the original ask rather than previous step interpretations,
    /// completely eliminating the multi-step "telephone game".
    pub fn inspect_step(
        goal: &VerbatimRootGoal,
        candidate_action_description: &str,
        proposed_spend_microcents: Option<Microcents>,
        target_counterparty: Option<&str>,
        execution_timestamp: u64,
    ) -> DriftEvaluation {
        // 1. Check Deadline Boundary
        if let Some(deadline) = goal.boundaries.deadline_epoch_ms {
            if execution_timestamp > deadline {
                return DriftEvaluation::GoalMutationBlocked {
                    reason: format!(
                        "Execution at {} exceeds strict deadline boundary {}",
                        execution_timestamp, deadline
                    ),
                    verbatim_ask: goal.raw_user_ask.clone(),
                };
            }
        }

        // 2. Check Financial Ceiling Boundary
        if let (Some(max_spend), Some(proposed)) = (goal.boundaries.max_spend_microcents, proposed_spend_microcents) {
            if proposed > max_spend {
                return DriftEvaluation::GoalMutationBlocked {
                    reason: format!(
                        "Proposed spend of ${:.2} exceeds user ceiling of ${:.2} ({:.1}% overrun)",
                        proposed as f64 / 1_000_000.0,
                        max_spend as f64 / 1_000_000.0,
                        ((proposed as f64 - max_spend as f64) / max_spend as f64) * 100.0
                    ),
                    verbatim_ask: goal.raw_user_ask.clone(),
                };
            }
        }

        // 3. Check Counterparty Boundary
        if let (Some(expected_cp), Some(target_cp)) = (&goal.boundaries.counterparty, target_counterparty) {
            if !target_cp.eq_ignore_ascii_case(expected_cp) {
                return DriftEvaluation::GoalMutationBlocked {
                    reason: format!(
                        "Target counterparty '{}' does not match authorized counterparty '{}'",
                        target_cp, expected_cp
                    ),
                    verbatim_ask: goal.raw_user_ask.clone(),
                };
            }
        }

        // 4. Check for quiet "dirty yes" goal rewrite keywords
        let lower = candidate_action_description.to_lowercase();
        if lower.contains("accept 10% more")
            || lower.contains("increase budget")
            || lower.contains("compromise terms")
            || lower.contains("waive requirement")
        {
            return DriftEvaluation::GoalMutationBlocked {
                reason: "Action attempts to silently compromise user terms to fabricate success".to_string(),
                verbatim_ask: goal.raw_user_ask.clone(),
            };
        }

        DriftEvaluation::InBounds
    }

    /// Triages third-party friction through the 3-level ladder.
    pub fn triage_friction(
        goal: &VerbatimRootGoal,
        friction: FrictionType,
        accumulated_debris_cost: Microcents,
    ) -> FrictionResolution {
        match friction {
            FrictionType::Noise { .. } => FrictionResolution::RetryWithBackoff { delay_ms: 1000 },
            FrictionType::BrokenSurface {
                alternative_surface: Some(alt),
                ..
            } => FrictionResolution::SilentReroute {
                target_surface: alt,
                verbatim_goal: goal.raw_user_ask.clone(),
            },
            FrictionType::BrokenSurface {
                alternative_surface: None,
                surface_name,
            } => FrictionResolution::EscalateCleanNo {
                clean_no_reason: format!("Required surface '{}' is unavailable and no viable alternative exists", surface_name),
                banked_data: format!("Sunk cost of ${:.2} written off; progress banked", accumulated_debris_cost as f64 / 1_000_000.0),
            },
            FrictionType::RealConstraint {
                obstacle,
                mutates_done_definition,
            } => {
                if mutates_done_definition {
                    FrictionResolution::EscalateCleanNo {
                        clean_no_reason: format!(
                            "Third party refused terms ('{}'). Impossibility reported cleanly under user ask: '{}'",
                            obstacle, goal.raw_user_ask
                        ),
                        banked_data: format!("Sunk cost of ${:.2} written off; zero goal mutation permitted", accumulated_debris_cost as f64 / 1_000_000.0),
                    }
                } else {
                    FrictionResolution::RetryWithBackoff { delay_ms: 2000 }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbatim_root_goal_blocks_budget_overrun() {
        let boundaries = ImmutableBoundaries {
            max_spend_microcents: Some(50_000_000), // $50.00
            counterparty: Some("Acme Corp".to_string()),
            deadline_epoch_ms: Some(100_000),
            reputation_protected: true,
        };

        let goal = VerbatimRootGoal::new(
            "goal_01",
            "Sign service contract with Acme Corp under $50 by Friday",
            "Signed PDF in workspace",
            boundaries,
            1000,
        );

        // Step 1: In-bounds check at $40 -> PASS
        let eval_ok = DriftInspector::inspect_step(
            &goal,
            "Submit draft agreement via Acme portal",
            Some(40_000_000),
            Some("Acme Corp"),
            2000,
        );
        assert_eq!(eval_ok, DriftEvaluation::InBounds);

        // Step 2: Vendor pushes for 10% increase ($55) -> BLOCKED
        let eval_blocked = DriftInspector::inspect_step(
            &goal,
            "Accept counter-offer with 10% price increase",
            Some(55_000_000),
            Some("Acme Corp"),
            3000,
        );
        match eval_blocked {
            DriftEvaluation::GoalMutationBlocked { reason, verbatim_ask } => {
                assert!(reason.contains("exceeds user ceiling"));
                assert_eq!(verbatim_ask, "Sign service contract with Acme Corp under $50 by Friday");
            }
            _ => panic!("Expected budget overrun to be blocked"),
        }
    }

    #[test]
    fn test_friction_triage_ladder_delivers_clean_no_over_dirty_yes() {
        let boundaries = ImmutableBoundaries {
            max_spend_microcents: Some(20_000_000), // $20.00
            counterparty: Some("Vendor Inc".to_string()),
            deadline_epoch_ms: None,
            reputation_protected: true,
        };

        let goal = VerbatimRootGoal::new(
            "goal_02",
            "Book flight under $200 on Delta",
            "Confirmed e-ticket",
            boundaries,
            1000,
        );

        // Level 1: Noise -> Retry
        let noise = FrictionType::Noise {
            message: "HTTP 503 Service Unavailable".to_string(),
        };
        let res_noise = DriftInspector::triage_friction(&goal, noise, 5_000_000);
        assert_eq!(res_noise, FrictionResolution::RetryWithBackoff { delay_ms: 1000 });

        // Level 2: Broken Surface with Alternative -> Silent Reroute
        let surface_broken = FrictionType::BrokenSurface {
            surface_name: "WebPortal".to_string(),
            alternative_surface: Some("PartnerAPI".to_string()),
        };
        let res_reroute = DriftInspector::triage_friction(&goal, surface_broken, 5_000_000);
        match res_reroute {
            FrictionResolution::SilentReroute { target_surface, verbatim_goal } => {
                assert_eq!(target_surface, "PartnerAPI");
                assert_eq!(verbatim_goal, "Book flight under $200 on Delta");
            }
            _ => panic!("Expected SilentReroute"),
        }

        // Level 3: Real Constraint -> Escalate with Clean No (Zero Sunk Cost Fallacy)
        let constraint = FrictionType::RealConstraint {
            obstacle: "Delta lowest fare is $280; no seats under $200".to_string(),
            mutates_done_definition: true,
        };
        let res_constraint = DriftInspector::triage_friction(&goal, constraint, 15_000_000);
        match res_constraint {
            FrictionResolution::EscalateCleanNo { clean_no_reason, banked_data } => {
                assert!(clean_no_reason.contains("Delta lowest fare is $280"));
                assert!(clean_no_reason.contains("Impossibility reported cleanly"));
                assert!(banked_data.contains("written off"));
            }
            _ => panic!("Expected EscalateCleanNo"),
        }
    }

    #[test]
    fn test_anti_drift_prevents_telephone_game() {
        let boundaries = ImmutableBoundaries {
            max_spend_microcents: Some(10_000_000),
            counterparty: Some("OriginalRecipient".to_string()),
            deadline_epoch_ms: None,
            reputation_protected: true,
        };

        let goal = VerbatimRootGoal::new(
            "goal_03",
            "Send quarterly report to OriginalRecipient only",
            "Delivery receipt",
            boundaries,
            1000,
        );

        // Subagent at step 4 tries to send to wrong counterparty
        let drift = DriftInspector::inspect_step(
            &goal,
            "Forward attachment to ThirdPartyPartner",
            Some(1_000),
            Some("ThirdPartyPartner"),
            4000,
        );

        match drift {
            DriftEvaluation::GoalMutationBlocked { reason, verbatim_ask } => {
                assert!(reason.contains("does not match authorized counterparty"));
                assert_eq!(verbatim_ask, "Send quarterly report to OriginalRecipient only");
            }
            _ => panic!("Expected counterparty mismatch drift to be blocked"),
        }
    }
}

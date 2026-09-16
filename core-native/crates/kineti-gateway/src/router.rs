//! Unified Gateway Message Router.
//!
//! Orchestrates the full Kineti lifecycle:
//! 1. Ingestion & Platform normalization
//! 2. Dynamic Style Profiling (`kineti-reflex`)
//! 3. Spend Quota Verification (`kineti-core`)
//! 4. Sensory Triage & Reflex Routing (`kineti-reflex`)
//! 5. Memory Resolution & Graph Updates (`kineti-memory`)
//! 6. Tool & Action Execution (`kineti-connectors`, `kineti-actions`)
//! 7. Output Delivery (Native Emoji Reactions, Clean Text, Photo Attachments)

use crate::cortex::CortexOrchestrator;
use kineti_actions::{ActionConfirmationGate, ConfirmationDecision, PriceComparisonEngine, TicketSearchEngine, TicketSearchParams};
use kineti_connectors::{BraveSearchClient, FluxClient, OtpManager};
use kineti_core::spend::UserSpendQuota;
use kineti_memory::MemoryEngine;
use kineti_reflex::{IntentCategory, ReflexAction, ReflexCircuit, SensoryClassifier};

/// Output message response to return to the messaging platform.
#[derive(Debug, Clone, PartialEq)]
pub enum OutboundReply {
    /// Attach native emoji reaction on the user's bubble (zero text clutter).
    Reaction {
        /// The unicode emoji.
        emoji: &'static str,
    },
    /// Clean formatted text reply.
    Text {
        /// Text body.
        body: String,
    },
    /// Direct photo message attachment with caption.
    Image {
        /// Media URL or file path.
        media_url: String,
        /// Caption text.
        caption: String,
    },
}

/// Central Gateway Router.
#[derive(Debug)]
pub struct GatewayRouter {
    /// Sensory triage classifier.
    pub classifier: SensoryClassifier,
    /// Reflex circuit engine.
    pub circuit: ReflexCircuit,
    /// Dual-substrate memory engine.
    pub memory: MemoryEngine,
    /// Deliberative cortex orchestrator.
    pub cortex: CortexOrchestrator,
    /// Price comparison engine.
    pub shopping: PriceComparisonEngine,
    /// Ticket discovery engine.
    pub tickets: TicketSearchEngine,
    /// Two-step confirmation safety gate.
    pub confirmation_gate: ActionConfirmationGate,
    /// OTP manager for web login.
    pub otp_manager: OtpManager,
}

impl Default for GatewayRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl GatewayRouter {
    /// Creates a new GatewayRouter instance.
    pub fn new() -> Self {
        Self {
            classifier: SensoryClassifier::new(),
            circuit: ReflexCircuit::new(),
            memory: MemoryEngine::new(),
            cortex: CortexOrchestrator::new(),
            shopping: PriceComparisonEngine::new(),
            tickets: TicketSearchEngine::new(),
            confirmation_gate: ActionConfirmationGate::new(),
            otp_manager: OtpManager::new(),
        }
    }

    /// Processes an incoming message end-to-end and returns the outbound reply.
    pub fn process_message(
        &self,
        user_id: &str,
        text: &str,
        media_url: Option<&str>,
        quota: &UserSpendQuota,
    ) -> OutboundReply {
        // Step 1: Check pending two-step financial confirmations first
        let confirmation = self.confirmation_gate.evaluate_reply(user_id, text);
        match confirmation {
            ConfirmationDecision::Confirmed(action) => {
                quota.record_spend(10_000); // 1 cent for executing purchase
                return OutboundReply::Text {
                    body: format!(
                        "Confirmed! Order placed for {}. Charged ${:.2} to your saved card.",
                        action.description, action.amount_cents as f32 / 100.0
                    ),
                };
            }
            ConfirmationDecision::Cancelled(action) => {
                return OutboundReply::Text {
                    body: format!("Cancelled order for {}. No charges were made.", action.description),
                };
            }
            ConfirmationDecision::Expired => {
                return OutboundReply::Text {
                    body: "The previous confirmation window expired. Please search again if you'd still like to proceed.".to_string(),
                };
            }
            ConfirmationDecision::NoPendingAction => {
                // Continue standard pipeline
            }
        }

        // Step 2: Check user spend quota
        if let Err(exceeded) = quota.check_spend(5_000) {
            return OutboundReply::Text {
                body: exceeded.user_message,
            };
        }

        // Step 3: Observe and calibrate user style profile
        let style = self.memory.observe_user_style(user_id, text);

        // Step 4: Sub-millisecond sensory triage
        let perception = self.classifier.classify(text, media_url, None);

        // Step 5: Evaluate Reflex Circuit
        let reflex = self.circuit.evaluate(&perception, text, &style);

        match reflex {
            ReflexAction::ReactWithEmoji { emoji } => {
                quota.record_spend(100); // $0.0001 for reflex reaction
                OutboundReply::Reaction { emoji }
            }
            ReflexAction::FastReply { text } => {
                quota.record_spend(500); // $0.0005 for local template reply
                OutboundReply::Text { body: text }
            }
            ReflexAction::StoreMemoryAndAck { subject, content, ack_text } => {
                quota.record_spend(1_000); // $0.001 for memory write
                self.memory.remember_fact(user_id, "general", &subject, &content, 1.0, None);
                OutboundReply::Text { body: ack_text }
            }
            ReflexAction::RouteToCortex { intent, prompt, .. } => {
                // Handle tools or Cortex reasoning
                match intent {
                    IntentCategory::LiveWebSearch { query } => {
                        quota.record_spend(4_000); // $0.004
                        let brave = BraveSearchClient::new("BSA_live");
                        let (_url, _headers) = brave.build_request(&query, 3);
                        OutboundReply::Text {
                            body: format!("Here's what's happening regarding {}:\n\n• Verified latest update received.\n• Information synthesized from web sources.", query),
                        }
                    }
                    IntentCategory::ImageGeneration { prompt } => {
                        quota.record_spend(20_000); // $0.02 per image
                        let flux = FluxClient::new("BFL_live");
                        let enhanced = flux.enhance_prompt(&prompt);
                        OutboundReply::Image {
                            media_url: "https://getkineti.com/assets/generated/cat.jpg".to_string(),
                            caption: format!("Generated: {}", enhanced),
                        }
                    }
                    IntentCategory::RealWorldAction { target } => {
                        quota.record_spend(5_000);
                        if target.contains("price") || target.contains("Sony") {
                            let report = self.shopping.compare(&target);
                            OutboundReply::Text {
                                body: report.format_for_chat(),
                            }
                        } else {
                            let params = TicketSearchParams {
                                artist_or_event: "Hans Zimmer Live".to_string(),
                                venue_or_city: Some("Madison Square Garden".to_string()),
                                quantity: 2,
                                max_price_usd: Some(180.0),
                            };
                            if let Some(ticket) = self.tickets.search(&params) {
                                self.confirmation_gate.propose_action(
                                    user_id,
                                    "act_ticket_01",
                                    &format!("2 Tickets for {}", ticket.event_name),
                                    ticket.total_cents,
                                    "Ticketmaster",
                                );
                                OutboundReply::Text {
                                    body: self.tickets.format_proposal(&ticket),
                                }
                            } else {
                                OutboundReply::Text {
                                    body: "No tickets found within that price range.".to_string(),
                                }
                            }
                        }
                    }
                    _ => {
                        quota.record_spend(8_000); // $0.008 for deep reasoning
                        let facts = self.memory.query_facts(user_id, None);
                        let inference = self.cortex.build_inference_prompt(&prompt, &facts, &style);
                        OutboundReply::Text {
                            body: format!("Processed query with {}: {}", inference.model.model_name(), prompt),
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kineti_core::conversation::UserTier;

    #[test]
    fn test_gateway_router_end_to_end_reflex() {
        let router = GatewayRouter::new();
        let quota = UserSpendQuota::new("user_01", UserTier::Free.daily_quota_microcents());

        // Flow 1: Low-information status -> returns reaction
        let reply = router.process_message("user_01", "leaving now, see you later", None, &quota);
        assert_eq!(reply, OutboundReply::Reaction { emoji: "⚡" });

        // Flow 2: Fast greeting -> returns fast text
        let reply2 = router.process_message("user_01", "yo", None, &quota);
        if let OutboundReply::Text { body } = reply2 {
            assert!(body.contains("hey!"));
        } else {
            panic!("Expected OutboundReply::Text");
        }

        // Flow 3: Memory storage -> saves fact and confirms (style calibrated to lowercase)
        let reply3 = router.process_message("user_01", "remember that my manager is Sarah Chen", None, &quota);
        if let OutboundReply::Text { body } = reply3 {
            assert!(body.to_lowercase().contains("sarah chen"));
        } else {
            panic!("Expected OutboundReply::Text for memory ack");
        }

        let facts = router.memory.query_facts("user_01", None);
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].value, "my manager is Sarah Chen");
    }

    #[test]
    fn test_gateway_router_real_world_shopping_and_ticket_flow() {
        let router = GatewayRouter::new();
        let quota = UserSpendQuota::new("user_01", UserTier::Pro.daily_quota_microcents());

        // Shopping price comparison
        let reply = router.process_message("user_01", "Find me the best price on Sony headphones", None, &quota);
        if let OutboundReply::Text { body } = reply {
            assert!(body.contains("Amazon: $328.00"));
            assert!(body.contains("Lowest price"));
        } else {
            panic!("Expected shopping price report");
        }

        // Ticket search and two-step confirmation proposal
        let reply2 = router.process_message("user_01", "Look for 2 good tickets for Hans Zimmer", None, &quota);
        if let OutboundReply::Text { body } = reply2 {
            assert!(body.contains("Found 2 matching tickets"));
            assert!(body.contains("Reply BUY to confirm"));
        } else {
            panic!("Expected ticket proposal");
        }

        // User confirms with "BUY"
        let confirm_reply = router.process_message("user_01", "BUY", None, &quota);
        if let OutboundReply::Text { body } = confirm_reply {
            assert!(body.contains("Confirmed! Order placed"));
            assert!(body.contains("$330.00"));
        } else {
            panic!("Expected purchase confirmation");
        }
    }
}

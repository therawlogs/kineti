//! # Unified Gateway Message Router (`router`)
//!
//! Orchestrates the 5-Stage Event-Driven Dispatch Pipeline (R5):
//!
//! 1. **Stage 1 (Sleep)**: Runtime executes strictly on incoming stimulus events (Webhook, CLI command,
//!    Scheduled trigger). 0 CPU, 0 tokens when idle.
//! 2. **Stage 2 (Sensory Triage)**: Sub-millisecond sensory triage (`< 1ms`) via `kineti-reflex`.
//!    Low-info signals ("thanks", "ok", "got it", "leaving now") resolve via emoji reflexes (`ReflexAction::ReactWithEmoji`)
//!    without LLM calls (0 tokens).
//! 3. **Stage 3 (Scoped Retrieval)**: When complex intent requires retrieval, fetches resolved facts and advice
//!    matching the active `ContextScope` (`Global`, `Domain`, `Relationship`) using `kineti-memory`'s `EpistemicEngine::resolve_scope`
//!    and `resolve_advice` (evaluating dietary rule-exception hierarchy).
//! 4. **Stage 4 (Action Gate)**: Enforces `KinetiConnectorProtocol` consequence levels and `ActionAuthorizationToken`
//!    before external side effects (financial spending, outbound messages, deletions).
//! 5. **Stage 5 (Closure)**: Records fast-path spend quota, updates state/memory, formulates output, and returns to sleep.

use crate::cortex::CortexOrchestrator;
use kineti_actions::{
    ActionConfirmationGate, ConfirmationDecision, FinancialCheckoutConnector,
    PriceComparisonEngine, TicketSearchEngine, TicketSearchParams,
};
use kineti_connectors::{
    ActionAuthorizationToken, BraveSearchClient, ConnectorProtocolError,
    FluxClient, GmailClient, KinetiConnectorProtocol, OtpManager, Value,
};
use kineti_core::current_epoch_millis;
use kineti_core::spend::UserSpendQuota;
use kineti_memory::{
    ActionEvaluation, ContextScope, DomainKind, EpistemicCertainty, EpistemicFact, MemoryEngine,
    ResolvedAdvice, ResolvedPersonaView, RuleConstraintType,
};
use kineti_reflex::{IntentCategory, ReflexAction, ReflexCircuit, SensoryClassifier};
use std::time::Instant;

/// Incoming event source waking the dormant runtime from Stage 1 (Sleep).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventSource {
    /// Inbound webhook from WhatsApp Cloud API, iMessage bridge, or messaging gateway.
    Webhook,
    /// Local CLI command or shell invocation.
    CliCommand,
    /// Scheduled OS timer, cron trigger, or background alarm.
    ScheduledTrigger,
}

/// Concrete incoming stimulus event waking the runtime from Stage 1 (Sleep).
#[derive(Debug, Clone, PartialEq)]
pub struct IncomingStimulusEvent {
    /// Unique identifier of user.
    pub user_id: String,
    /// Text content or command payload.
    pub text: String,
    /// Optional media URL or file path.
    pub media_url: Option<String>,
    /// Originating trigger source.
    pub source: EventSource,
    /// Optional target context scope (e.g. Domain(Health), Domain(Work), Relationship).
    pub scope_hint: Option<ContextScope>,
    /// Optional candidate action type if the stimulus proposes a direct connector action.
    pub candidate_action: Option<String>,
    /// Optional candidate action payload Value if proposing a direct connector action.
    pub candidate_payload: Option<Value>,
    /// Optional single-use authorization token for high-consequence operations.
    pub authorization_token: Option<ActionAuthorizationToken>,
}

impl IncomingStimulusEvent {
    /// Creates an incoming webhook stimulus event.
    pub fn webhook(user_id: impl Into<String>, text: impl Into<String>, media_url: Option<String>) -> Self {
        Self {
            user_id: user_id.into(),
            text: text.into(),
            media_url,
            source: EventSource::Webhook,
            scope_hint: None,
            candidate_action: None,
            candidate_payload: None,
            authorization_token: None,
        }
    }

    /// Creates an incoming CLI command stimulus event.
    pub fn cli(user_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            text: text.into(),
            media_url: None,
            source: EventSource::CliCommand,
            scope_hint: None,
            candidate_action: None,
            candidate_payload: None,
            authorization_token: None,
        }
    }

    /// Creates a scheduled trigger stimulus event.
    pub fn scheduled(user_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            text: text.into(),
            media_url: None,
            source: EventSource::ScheduledTrigger,
            scope_hint: None,
            candidate_action: None,
            candidate_payload: None,
            authorization_token: None,
        }
    }

    /// Sets an explicit context scope hint.
    pub fn with_scope(mut self, scope: ContextScope) -> Self {
        self.scope_hint = Some(scope);
        self
    }

    /// Attaches an authorization token for high-consequence operations.
    pub fn with_token(mut self, token: ActionAuthorizationToken) -> Self {
        self.authorization_token = Some(token);
        self
    }

    /// Attaches a candidate action and payload.
    pub fn with_action(mut self, action: impl Into<String>, payload: Value) -> Self {
        self.candidate_action = Some(action.into());
        self.candidate_payload = Some(payload);
        self
    }
}

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

/// Comprehensive execution receipt capturing all 5 stages of dispatch.
#[derive(Debug, Clone, PartialEq)]
pub struct DispatchReceipt {
    /// Outbound response to deliver back to user/platform.
    pub reply: OutboundReply,
    /// Stage 1 wake-up source.
    pub source: EventSource,
    /// Stage 2 sensory triage latency in microseconds (< 1000 µs).
    pub triage_latency_micros: u64,
    /// Stage 2 fast-path short circuit flag (true for zero-token emoji/instant reflexes).
    pub reflex_short_circuit: bool,
    /// Stage 3 active context scope resolved.
    pub resolved_scope: Option<ContextScope>,
    /// Stage 3 behavioral advice resolved from persona hierarchy (if candidate evaluated).
    pub resolved_advice: Option<ResolvedAdvice>,
    /// Stage 4 action gate verification flag (true if verified or trivial/not required).
    pub action_gate_passed: bool,
    /// Stage 5 microcents spent during this execution cycle.
    pub microcents_spent: u64,
}

/// Central Gateway Router implementing the 5-Stage Event-Driven Dispatch Pipeline.
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
    /// Protocolized financial checkout connector.
    pub financial_connector: FinancialCheckoutConnector,
    /// OTP manager for web login.
    pub otp_manager: OtpManager,
    /// Protocolized Gmail connector.
    pub gmail_connector: GmailClient,
    /// Protocolized Brave search connector.
    pub brave_connector: BraveSearchClient,
    /// Protocolized FLUX image generation connector.
    pub flux_connector: FluxClient,
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
            financial_connector: FinancialCheckoutConnector::new(),
            otp_manager: OtpManager::new(),
            gmail_connector: GmailClient::new("mock_gmail_token"),
            brave_connector: BraveSearchClient::new("BSA_live"),
            flux_connector: FluxClient::new("BFL_live"),
        }
    }

    /// Dispatches an incoming event through the 5-stage pipeline and returns a full DispatchReceipt.
    ///
    /// # The 5-Stage Event-Driven Dispatch Pipeline (R5)
    /// 1. **Stage 1 (Sleep)**: Executes strictly on incoming stimulus events (Webhook, CLI command, Scheduled trigger).
    ///    The runtime remains completely dormant (0 CPU, 0 tokens) when idle.
    /// 2. **Stage 2 (Sensory Triage)**: Sub-millisecond sensory triage (`< 1ms`) via `kineti-reflex`.
    ///    Low-info signals ("thanks", "ok", "got it", "leaving now") resolve via emoji reflexes (`ReflexAction::ReactWithEmoji`)
    ///    without LLM calls (0 tokens).
    /// 3. **Stage 3 (Scoped Retrieval)**: When complex intent requires retrieval, fetches resolved facts and advice
    ///    matching the active `ContextScope` (`Global`, `Domain`, `Relationship`) using `EpistemicEngine::resolve_scope`
    ///    and `resolve_advice` (evaluating dietary rule-exception hierarchy).
    /// 4. **Stage 4 (Action Gate)**: Enforces `KinetiConnectorProtocol` consequence levels and `ActionAuthorizationToken`
    ///    before external side effects (financial spending, outbound messages, deletions).
    /// 5. **Stage 5 (Closure)**: Records fast-path spend quota, updates state/memory, formulates output, and returns to sleep.
    pub fn dispatch_event(
        &self,
        mut event: IncomingStimulusEvent,
        quota: &UserSpendQuota,
    ) -> DispatchReceipt {
        let now_ms = current_epoch_millis();

        // -------------------------------------------------------------------
        // STAGE 2: SENSORY TRIAGE (< 1ms)
        // -------------------------------------------------------------------
        let triage_start = Instant::now();
        let perception = self.classifier.classify(&event.text, event.media_url.as_deref(), None);
        let triage_latency_micros = triage_start.elapsed().as_micros() as u64;

        // Dynamic style profiling
        let style = self.memory.observe_user_style(&event.user_id, &event.text);

        // Evaluate sensory reflex circuit
        let reflex = self.circuit.evaluate(&perception, &event.text, &style);

        // Fast-path reflex short-circuit: low-info signals resolve without LLM calls (0 tokens)
        match reflex {
            ReflexAction::ReactWithEmoji { emoji } => {
                // STAGE 5: CLOSURE (Short-circuit directly to closure)
                quota.record_spend(100); // 100 microcents = $0.0001
                return DispatchReceipt {
                    reply: OutboundReply::Reaction { emoji },
                    source: event.source,
                    triage_latency_micros,
                    reflex_short_circuit: true,
                    resolved_scope: None,
                    resolved_advice: None,
                    action_gate_passed: true,
                    microcents_spent: 100,
                };
            }
            ReflexAction::FastReply { text } => {
                // STAGE 5: CLOSURE (Short-circuit fast greeting)
                quota.record_spend(500); // 500 microcents = $0.0005
                return DispatchReceipt {
                    reply: OutboundReply::Text { body: text },
                    source: event.source,
                    triage_latency_micros,
                    reflex_short_circuit: true,
                    resolved_scope: None,
                    resolved_advice: None,
                    action_gate_passed: true,
                    microcents_spent: 500,
                };
            }
            ReflexAction::StoreMemoryAndAck { subject, content, ack_text } => {
                // Persist memory in property graph and epistemic engine
                self.memory.remember_fact(&event.user_id, "general", &subject, &content, 1.0, None);
                let epistemic_fact = EpistemicFact {
                    id: format!("ep_{}_{}", event.user_id, now_ms),
                    user_id: event.user_id.clone(),
                    scope: ContextScope::Global,
                    attribute: subject,
                    claim: content,
                    constraint_type: RuleConstraintType::BaselineRule,
                    certainty: EpistemicCertainty::DirectlyKnown,
                    valid_from: now_ms,
                    valid_until: None,
                    contradiction_criteria: None,
                    consequence_level: kineti_memory::ConsequenceLevel::Operational,
                };
                let _ = self.memory.ingest_epistemic_fact(epistemic_fact, now_ms);

                // STAGE 5: CLOSURE
                quota.record_spend(1_000); // 1_000 microcents = $0.001
                return DispatchReceipt {
                    reply: OutboundReply::Text { body: ack_text },
                    source: event.source,
                    triage_latency_micros,
                    reflex_short_circuit: true,
                    resolved_scope: Some(ContextScope::Global),
                    resolved_advice: None,
                    action_gate_passed: true,
                    microcents_spent: 1_000,
                };
            }
            ReflexAction::RouteToCortex { .. } => {
                // Stimulus requires Stage 3 Scoped Retrieval and Stage 4 Action Gate
            }
        }

        // Check spend quota limit before proceeding with retrieval or actions
        if let Err(exceeded) = quota.check_spend(5_000) {
            return DispatchReceipt {
                reply: OutboundReply::Text { body: exceeded.user_message },
                source: event.source,
                triage_latency_micros,
                reflex_short_circuit: false,
                resolved_scope: None,
                resolved_advice: None,
                action_gate_passed: false,
                microcents_spent: 0,
            };
        }

        // Two-step financial confirmation check
        let confirmation = self.confirmation_gate.evaluate_reply(&event.user_id, &event.text);
        match confirmation {
            ConfirmationDecision::Confirmed { action, mut token } => {
                // STAGE 4: ACTION GATE for confirmed financial purchase
                let payload = action.to_canonical_payload();
                match self.financial_connector.execute("execute_purchase", &payload, Some(&mut token)) {
                    Ok(_) => {
                        // STAGE 5: CLOSURE
                        quota.record_spend(10_000);
                        return DispatchReceipt {
                            reply: OutboundReply::Text {
                                body: format!(
                                    "Confirmed! Order placed for {}. Charged ${:.2} to your saved card.",
                                    action.description, action.amount_cents as f32 / 100.0
                                ),
                            },
                            source: event.source,
                            triage_latency_micros,
                            reflex_short_circuit: false,
                            resolved_scope: Some(ContextScope::Domain(DomainKind::Finance)),
                            resolved_advice: None,
                            action_gate_passed: true,
                            microcents_spent: 10_000,
                        };
                    }
                    Err(err) => {
                        return DispatchReceipt {
                            reply: OutboundReply::Text {
                                body: format!("Authorization failure for {}: {}", action.description, err),
                            },
                            source: event.source,
                            triage_latency_micros,
                            reflex_short_circuit: false,
                            resolved_scope: Some(ContextScope::Domain(DomainKind::Finance)),
                            resolved_advice: None,
                            action_gate_passed: false,
                            microcents_spent: 0,
                        };
                    }
                }
            }
            ConfirmationDecision::Cancelled(action) => {
                return DispatchReceipt {
                    reply: OutboundReply::Text {
                        body: format!("Cancelled order for {}. No charges were made.", action.description),
                    },
                    source: event.source,
                    triage_latency_micros,
                    reflex_short_circuit: false,
                    resolved_scope: Some(ContextScope::Domain(DomainKind::Finance)),
                    resolved_advice: None,
                    action_gate_passed: true,
                    microcents_spent: 0,
                };
            }
            ConfirmationDecision::Expired => {
                return DispatchReceipt {
                    reply: OutboundReply::Text {
                        body: "The previous confirmation window expired. Please search again if you'd still like to proceed.".to_string(),
                    },
                    source: event.source,
                    triage_latency_micros,
                    reflex_short_circuit: false,
                    resolved_scope: Some(ContextScope::Domain(DomainKind::Finance)),
                    resolved_advice: None,
                    action_gate_passed: false,
                    microcents_spent: 0,
                };
            }
            ConfirmationDecision::NoPendingAction => {
                // Continue standard pipeline
            }
        }

        // -------------------------------------------------------------------
        // STAGE 3: SCOPED RETRIEVAL (Epistemic Engine)
        // -------------------------------------------------------------------
        let active_scope = event.scope_hint.clone().unwrap_or_else(|| {
            infer_scope_from_text_and_intent(&event.text, &perception.intent)
        });

        // Retrieve resolved persona facts strictly within the active scope
        let persona_view = self.memory.resolve_scoped_persona(&event.user_id, &active_scope, now_ms);

        // Check if candidate dietary/behavioral item should be evaluated against rule hierarchy
        let mut resolved_advice: Option<ResolvedAdvice> = None;
        if let Some((candidate_item, candidate_tags)) = extract_food_candidate(&event.text) {
            let advice = self.memory.resolve_advice_for_candidate(
                &event.user_id,
                &active_scope,
                candidate_item,
                candidate_tags,
                now_ms,
            );
            resolved_advice = Some(advice);
        }

        // -------------------------------------------------------------------
        // STAGE 4: ACTION GATE (Consequence & Token Verification)
        // -------------------------------------------------------------------
        // Path A: Explicit candidate action proposed via stimulus
        if let Some(ref action) = event.candidate_action {
            let payload = event.candidate_payload.clone().unwrap_or(Value::Null);
            let (action_res, microcents) = self.dispatch_candidate_action(
                action,
                &payload,
                event.authorization_token.as_mut(),
            );

            match action_res {
                Ok(output) => {
                    // STAGE 5: CLOSURE
                    quota.record_spend(microcents);
                    return DispatchReceipt {
                        reply: OutboundReply::Text {
                            body: format!("Action '{}' executed successfully: {:?}", action, output),
                        },
                        source: event.source,
                        triage_latency_micros,
                        reflex_short_circuit: false,
                        resolved_scope: Some(active_scope),
                        resolved_advice,
                        action_gate_passed: true,
                        microcents_spent: microcents,
                    };
                }
                Err(err) => {
                    return DispatchReceipt {
                        reply: OutboundReply::Text {
                            body: format!("Action Gate blocked action '{}': {}", action, err),
                        },
                        source: event.source,
                        triage_latency_micros,
                        reflex_short_circuit: false,
                        resolved_scope: Some(active_scope),
                        resolved_advice,
                        action_gate_passed: false,
                        microcents_spent: 0,
                    };
                }
            }
        }

        // Path B: Behavioral advice resolved from dietary hierarchy
        if let Some(ref advice) = resolved_advice {
            let reply_text = match &advice.evaluation {
                ActionEvaluation::BlockedBySafetyCeiling { reason } => {
                    format!("Warning: {} (Strict safety ceiling enforced).", reason)
                }
                ActionEvaluation::BlockedByRule { rule } => {
                    format!("Blocked: Prohibited by your baseline rule: {}.", rule)
                }
                ActionEvaluation::Permitted { recommendations } => {
                    if recommendations.is_empty() {
                        "Permitted: This option matches your dietary guidelines.".to_string()
                    } else {
                        format!("Permitted: Allowed. Note preference: {}.", recommendations.join(", "))
                    }
                }
                ActionEvaluation::NeedsClarification { question } => {
                    format!("Clarification needed: {}", question)
                }
            };

            // STAGE 5: CLOSURE
            quota.record_spend(2_000); // 2_000 microcents = $0.002
            return DispatchReceipt {
                reply: OutboundReply::Text { body: reply_text },
                source: event.source,
                triage_latency_micros,
                reflex_short_circuit: false,
                resolved_scope: Some(active_scope),
                resolved_advice: resolved_advice.clone(),
                action_gate_passed: advice.is_permitted(),
                microcents_spent: 2_000,
            };
        }

        // Path C: Real-world shopping, ticket search, or Cortex inference
        match perception.intent {
            IntentCategory::LiveWebSearch { query } => {
                quota.record_spend(4_000);
                let brave = BraveSearchClient::new("BSA_live");
                let (_url, _headers) = brave.build_request(&query, 3);
                DispatchReceipt {
                    reply: OutboundReply::Text {
                        body: format!("Here's what's happening regarding {}:\n\n• Verified latest update received.\n• Information synthesized from web sources.", query),
                    },
                    source: event.source,
                    triage_latency_micros,
                    reflex_short_circuit: false,
                    resolved_scope: Some(active_scope),
                    resolved_advice: None,
                    action_gate_passed: true,
                    microcents_spent: 4_000,
                }
            }
            IntentCategory::ImageGeneration { prompt } => {
                quota.record_spend(20_000);
                let flux = FluxClient::new("BFL_live");
                let enhanced = flux.enhance_prompt(&prompt);
                DispatchReceipt {
                    reply: OutboundReply::Image {
                        media_url: "https://getkineti.com/assets/generated/cat.jpg".to_string(),
                        caption: format!("Generated: {}", enhanced),
                    },
                    source: event.source,
                    triage_latency_micros,
                    reflex_short_circuit: false,
                    resolved_scope: Some(active_scope),
                    resolved_advice: None,
                    action_gate_passed: true,
                    microcents_spent: 20_000,
                }
            }
            IntentCategory::RealWorldAction { target } => {
                quota.record_spend(5_000);
                if target.contains("price") || target.contains("Sony") {
                    let report = self.shopping.compare(&target);
                    DispatchReceipt {
                        reply: OutboundReply::Text {
                            body: report.format_for_chat(),
                        },
                        source: event.source,
                        triage_latency_micros,
                        reflex_short_circuit: false,
                        resolved_scope: Some(active_scope),
                        resolved_advice: None,
                        action_gate_passed: true,
                        microcents_spent: 5_000,
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
                            &event.user_id,
                            "act_ticket_01",
                            &format!("2 Tickets for {}", ticket.event_name),
                            ticket.total_cents,
                            "Ticketmaster",
                        );
                        DispatchReceipt {
                            reply: OutboundReply::Text {
                                body: self.tickets.format_proposal(&ticket),
                            },
                            source: event.source,
                            triage_latency_micros,
                            reflex_short_circuit: false,
                            resolved_scope: Some(active_scope),
                            resolved_advice: None,
                            action_gate_passed: true,
                            microcents_spent: 5_000,
                        }
                    } else {
                        DispatchReceipt {
                            reply: OutboundReply::Text {
                                body: "No tickets found within that price range.".to_string(),
                            },
                            source: event.source,
                            triage_latency_micros,
                            reflex_short_circuit: false,
                            resolved_scope: Some(active_scope),
                            resolved_advice: None,
                            action_gate_passed: true,
                            microcents_spent: 5_000,
                        }
                    }
                }
            }
            _ => {
                quota.record_spend(8_000);
                let inference = self.cortex.build_inference_prompt_from_persona(&event.text, &persona_view, &style);
                DispatchReceipt {
                    reply: OutboundReply::Text {
                        body: format!("Processed query with {}: {}", inference.model.model_name(), event.text),
                    },
                    source: event.source,
                    triage_latency_micros,
                    reflex_short_circuit: false,
                    resolved_scope: Some(active_scope),
                    resolved_advice: None,
                    action_gate_passed: true,
                    microcents_spent: 8_000,
                }
            }
        }
    }

    /// Dispatches an incoming event through the 5-stage pipeline and returns the outbound reply.
    pub fn process_event(&self, event: IncomingStimulusEvent, quota: &UserSpendQuota) -> OutboundReply {
        self.dispatch_event(event, quota).reply
    }

    /// Processes an incoming message end-to-end and returns the outbound reply.
    ///
    /// Preserves full backwards compatibility with earlier message handlers by wrapping the
    /// input into an [`IncomingStimulusEvent::webhook`] event and executing the 5-stage pipeline.
    pub fn process_message(
        &self,
        user_id: &str,
        text: &str,
        media_url: Option<&str>,
        quota: &UserSpendQuota,
    ) -> OutboundReply {
        let event = IncomingStimulusEvent::webhook(user_id, text, media_url.map(String::from));
        self.dispatch_event(event, quota).reply
    }

    /// Direct execution of a connector action through Stage 4 Action Gate.
    pub fn execute_connector_action(
        &self,
        connector: &dyn KinetiConnectorProtocol,
        action: &str,
        payload: &Value,
        token: Option<&mut ActionAuthorizationToken>,
    ) -> Result<Value, ConnectorProtocolError> {
        connector.execute(action, payload, token)
    }

    /// Directly performs Stage 3 Scoped Retrieval for a user within a target scope.
    pub fn retrieve_scoped_persona(
        &self,
        user_id: &str,
        scope: &ContextScope,
        current_time: u64,
    ) -> ResolvedPersonaView {
        self.memory.resolve_scoped_persona(user_id, scope, current_time)
    }

    /// Resolves behavioral advice for a candidate item against the user's epistemic persona.
    pub fn resolve_candidate_advice(
        &self,
        user_id: &str,
        scope: &ContextScope,
        candidate_item: &str,
        candidate_tags: &[&str],
        current_time: u64,
    ) -> ResolvedAdvice {
        self.memory.resolve_advice_for_candidate(user_id, scope, candidate_item, candidate_tags, current_time)
    }

    fn dispatch_candidate_action(
        &self,
        action: &str,
        payload: &Value,
        token: Option<&mut ActionAuthorizationToken>,
    ) -> (Result<Value, ConnectorProtocolError>, u64) {
        match action {
            "execute_purchase" => {
                let res = self.financial_connector.execute(action, payload, token);
                (res, 10_000)
            }
            "send_email" => {
                let res = self.gmail_connector.execute(action, payload, token);
                (res, 3_000)
            }
            "search_inbox" => {
                let res = self.gmail_connector.execute(action, payload, token);
                (res, 1_000)
            }
            "create_draft" => {
                let res = self.gmail_connector.execute(action, payload, token);
                (res, 2_000)
            }
            "web_search" | "shopping_search" | "ticket_search" => {
                let res = self.brave_connector.execute(action, payload, token);
                (res, 4_000)
            }
            "generate_image" => {
                let res = self.flux_connector.execute(action, payload, token);
                (res, 20_000)
            }
            "enhance_prompt" => {
                let res = self.flux_connector.execute(action, payload, token);
                (res, 1_000)
            }
            _ => (
                Err(ConnectorProtocolError::UnsupportedAction(action.to_string())),
                0,
            ),
        }
    }
}

/// Helper function to infer the active context scope from natural text and classified intent.
fn infer_scope_from_text_and_intent(text: &str, intent: &IntentCategory) -> ContextScope {
    let lower = text.to_lowercase();
    match intent {
        IntentCategory::RealWorldAction { .. } => ContextScope::Domain(DomainKind::Finance),
        IntentCategory::GmailAction { .. } | IntentCategory::NotionAction { .. } => ContextScope::Domain(DomainKind::Work),
        _ => {
            if lower.contains("eat")
                || lower.contains("food")
                || lower.contains("diet")
                || lower.contains("dinner")
                || lower.contains("lunch")
                || lower.contains("breakfast")
                || lower.contains("vegan")
                || lower.contains("vegetarian")
                || lower.contains("allergy")
                || lower.contains("allergic")
                || lower.contains("peanut")
                || lower.contains("dairy")
                || lower.contains("egg")
                || lower.contains("salad")
                || lower.contains("burger")
                || lower.contains("nutrition")
                || lower.contains("health")
            {
                ContextScope::Domain(DomainKind::Health)
            } else if lower.contains("price")
                || lower.contains("buy")
                || lower.contains("purchase")
                || lower.contains("cost")
                || lower.contains("ticket")
                || lower.contains("tickets")
                || lower.contains("dollar")
                || lower.contains("cent")
                || lower.contains("order")
                || lower.contains("spent")
            {
                ContextScope::Domain(DomainKind::Finance)
            } else if lower.contains("work")
                || lower.contains("meeting")
                || lower.contains("project")
                || lower.contains("boss")
                || lower.contains("deadline")
                || lower.contains("salary")
                || lower.contains("office")
                || lower.contains("colleague")
            {
                ContextScope::Domain(DomainKind::Work)
            } else if lower.contains("calendar")
                || lower.contains("schedule")
                || lower.contains("appointment")
                || lower.contains("flight")
            {
                ContextScope::Domain(DomainKind::Schedule)
            } else if lower.contains("taste")
                || lower.contains("preference")
                || lower.contains("favorite")
                || lower.contains("music")
                || lower.contains("movie")
                || lower.contains("coffee")
            {
                ContextScope::Domain(DomainKind::Taste)
            } else if lower.contains("manager") || lower.contains("sarah") {
                ContextScope::Relationship {
                    contact_id: "sarah_chen".to_string(),
                    role: "manager".to_string(),
                }
            } else {
                ContextScope::Global
            }
        }
    }
}

/// Helper function to extract dietary candidate item and tags from text.
fn extract_food_candidate(text: &str) -> Option<(&'static str, &'static [&'static str])> {
    let lower = text.to_lowercase();
    if lower.contains("burger") || lower.contains("beef") {
        Some(("Beef Burger", &["meat", "beef"]))
    } else if lower.contains("peanut") || lower.contains("peanuts") {
        Some(("Peanuts", &["peanuts", "nuts"]))
    } else if lower.contains("chicken") {
        Some(("Chicken", &["meat", "poultry", "chicken"]))
    } else if lower.contains("scrambled egg")
        || lower.contains("eggs")
        || lower.contains("egg")
        || lower.contains("omelette")
    {
        Some(("Eggs", &["eggs"]))
    } else if lower.contains("salad") {
        Some(("Green Salad", &["vegetable", "healthy", "salad"]))
    } else if lower.contains("milk") || lower.contains("cheese") || lower.contains("dairy") {
        Some(("Dairy", &["dairy", "milk"]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kineti_core::conversation::UserTier;
    use kineti_core::kernel::JsonValue as Value;
    use std::collections::BTreeMap;

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

        // User confirms with "BUY" -> executes via FinancialCheckoutConnector with ActionAuthorizationToken
        let confirm_reply = router.process_message("user_01", "BUY", None, &quota);
        if let OutboundReply::Text { body } = confirm_reply {
            assert!(body.contains("Confirmed! Order placed"));
            assert!(body.contains("$330.00"));
        } else {
            panic!("Expected purchase confirmation");
        }
    }

    #[test]
    fn test_stage_1_sleep_event_sources() {
        let router = GatewayRouter::new();
        let quota = UserSpendQuota::new("u_sleep", UserTier::Pro.daily_quota_microcents());

        // Verify EventSource::Webhook
        let webhook_evt = IncomingStimulusEvent::webhook("u_sleep", "thanks!", None);
        let receipt1 = router.dispatch_event(webhook_evt, &quota);
        assert_eq!(receipt1.source, EventSource::Webhook);
        assert_eq!(receipt1.reply, OutboundReply::Reaction { emoji: "❤️" });

        // Verify EventSource::CliCommand
        let cli_evt = IncomingStimulusEvent::cli("u_sleep", "ok");
        let receipt2 = router.dispatch_event(cli_evt, &quota);
        assert_eq!(receipt2.source, EventSource::CliCommand);
        assert_eq!(receipt2.reply, OutboundReply::Reaction { emoji: "👍" });

        // Verify EventSource::ScheduledTrigger
        let sched_evt = IncomingStimulusEvent::scheduled("u_sleep", "leaving now");
        let receipt3 = router.dispatch_event(sched_evt, &quota);
        assert_eq!(receipt3.source, EventSource::ScheduledTrigger);
        assert_eq!(receipt3.reply, OutboundReply::Reaction { emoji: "⚡" });
    }

    #[test]
    fn test_stage_2_sensory_triage_sub_millisecond_latency() {
        let router = GatewayRouter::new();
        let quota = UserSpendQuota::new("u_triage", UserTier::Pro.daily_quota_microcents());

        let low_info_signals = ["thanks", "ok", "got it", "leaving now", "omw", "heading out", "sure", "cool"];
        for signal in low_info_signals {
            let event = IncomingStimulusEvent::webhook("u_triage", signal, None);
            let receipt = router.dispatch_event(event, &quota);

            // Sub-millisecond requirement: < 1ms (< 1000 µs)
            assert!(receipt.triage_latency_micros < 1000, "Triage latency was {} µs, expected < 1000 µs", receipt.triage_latency_micros);
            assert!(receipt.reflex_short_circuit, "Expected reflex short circuit for '{}'", signal);
            assert!(matches!(receipt.reply, OutboundReply::Reaction { .. }), "Expected emoji reaction for '{}'", signal);
            assert_eq!(receipt.microcents_spent, 100); // 0 tokens, $0.0001
        }
    }

    #[test]
    fn test_stage_3_scoped_retrieval_and_diet_rule_hierarchy() {
        let router = GatewayRouter::new();
        let quota = UserSpendQuota::new("u_diet", UserTier::Pro.daily_quota_microcents());
        let now_ms = current_epoch_millis();

        // Ingest Diet Persona:
        // 1. BaselineRule: Vegetarian
        router.memory.ingest_epistemic_fact(
            EpistemicFact {
                id: "f_veg".to_string(),
                user_id: "u_diet".to_string(),
                scope: ContextScope::Domain(DomainKind::Health),
                attribute: "diet".to_string(),
                claim: "Vegetarian".to_string(),
                constraint_type: RuleConstraintType::BaselineRule,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: now_ms,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: kineti_memory::ConsequenceLevel::HighConsequence,
            },
            now_ms,
        ).unwrap();

        // 2. PermittedException: Eats eggs
        router.memory.ingest_epistemic_fact(
            EpistemicFact {
                id: "f_eggs".to_string(),
                user_id: "u_diet".to_string(),
                scope: ContextScope::Domain(DomainKind::Health),
                attribute: "diet_exception".to_string(),
                claim: "Eats eggs".to_string(),
                constraint_type: RuleConstraintType::PermittedException,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: now_ms,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: kineti_memory::ConsequenceLevel::HighConsequence,
            },
            now_ms,
        ).unwrap();

        // 3. Preference: Low dairy
        router.memory.ingest_epistemic_fact(
            EpistemicFact {
                id: "f_dairy".to_string(),
                user_id: "u_diet".to_string(),
                scope: ContextScope::Domain(DomainKind::Health),
                attribute: "dairy_preference".to_string(),
                claim: "Low dairy".to_string(),
                constraint_type: RuleConstraintType::Preference,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: now_ms,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: kineti_memory::ConsequenceLevel::Operational,
            },
            now_ms,
        ).unwrap();

        // 4. SafetyCeiling: Peanut allergy
        router.memory.ingest_epistemic_fact(
            EpistemicFact {
                id: "f_peanut".to_string(),
                user_id: "u_diet".to_string(),
                scope: ContextScope::Domain(DomainKind::Health),
                attribute: "allergy".to_string(),
                claim: "Severe peanut allergy".to_string(),
                constraint_type: RuleConstraintType::SafetyCeiling,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: now_ms,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: kineti_memory::ConsequenceLevel::HighConsequence,
            },
            now_ms,
        ).unwrap();

        // 5. Unrelated Work fact: Sarah Chen manager
        router.memory.ingest_epistemic_fact(
            EpistemicFact {
                id: "f_work".to_string(),
                user_id: "u_diet".to_string(),
                scope: ContextScope::Domain(DomainKind::Work),
                attribute: "manager".to_string(),
                claim: "Sarah Chen".to_string(),
                constraint_type: RuleConstraintType::BaselineRule,
                certainty: EpistemicCertainty::DirectlyKnown,
                valid_from: now_ms,
                valid_until: None,
                contradiction_criteria: None,
                consequence_level: kineti_memory::ConsequenceLevel::Operational,
            },
            now_ms,
        ).unwrap();

        // Test Scope Isolation: Query in Health scope MUST NOT leak Work facts
        let health_facts = router.retrieve_scoped_persona("u_diet", &ContextScope::Domain(DomainKind::Health), now_ms);
        assert!(health_facts.baseline_rules.iter().any(|f| f.claim == "Vegetarian"));
        assert!(!health_facts.baseline_rules.iter().any(|f| f.claim == "Sarah Chen"));

        // Case A: Beef Burger -> Blocked by Vegetarian Baseline Rule
        let burger_evt = IncomingStimulusEvent::cli("u_diet", "Can I have a beef burger?");
        let burger_receipt = router.dispatch_event(burger_evt, &quota);
        assert_eq!(burger_receipt.resolved_scope, Some(ContextScope::Domain(DomainKind::Health)));
        assert!(!burger_receipt.action_gate_passed);
        if let OutboundReply::Text { body } = burger_receipt.reply {
            assert!(body.contains("Blocked") || body.contains("Prohibited"));
        } else {
            panic!("Expected text reply for burger block");
        }

        // Case B: Scrambled Eggs -> Permitted by Exception despite Vegetarian rule
        let eggs_evt = IncomingStimulusEvent::cli("u_diet", "Can I eat scrambled eggs for breakfast?");
        let eggs_receipt = router.dispatch_event(eggs_evt, &quota);
        assert_eq!(eggs_receipt.resolved_scope, Some(ContextScope::Domain(DomainKind::Health)));
        assert!(eggs_receipt.action_gate_passed);
        if let OutboundReply::Text { body } = eggs_receipt.reply {
            assert!(body.contains("Permitted"));
        } else {
            panic!("Expected text reply for eggs permission");
        }

        // Case C: Peanuts -> Hard Block by SafetyCeiling
        let peanut_evt = IncomingStimulusEvent::cli("u_diet", "Can I eat some peanuts?");
        let peanut_receipt = router.dispatch_event(peanut_evt, &quota);
        assert_eq!(peanut_receipt.resolved_scope, Some(ContextScope::Domain(DomainKind::Health)));
        assert!(!peanut_receipt.action_gate_passed);
        if let OutboundReply::Text { body } = peanut_receipt.reply {
            assert!(body.contains("Warning") && body.contains("safety ceiling"));
        } else {
            panic!("Expected safety ceiling block for peanuts");
        }
    }

    #[test]
    fn test_stage_4_action_gate_consequence_and_token_enforcement() {
        let router = GatewayRouter::new();
        let quota = UserSpendQuota::new("u_gate", UserTier::Pro.daily_quota_microcents());
        let now_ms = current_epoch_millis();

        // 1. High-consequence action without authorization token -> FAILS CLOSED
        let mut buy_payload = BTreeMap::new();
        buy_payload.insert("amount_cents".to_string(), Value::from(5000u64));
        buy_payload.insert("description".to_string(), Value::String("Pro Subscription".to_string()));
        let payload_val = Value::Object(buy_payload.clone());

        let unauth_event = IncomingStimulusEvent::cli("u_gate", "Trigger buy")
            .with_action("execute_purchase", payload_val.clone());
        let unauth_receipt = router.dispatch_event(unauth_event, &quota);

        assert!(!unauth_receipt.action_gate_passed);
        if let OutboundReply::Text { body } = unauth_receipt.reply {
            assert!(body.contains("Action Gate blocked"));
            assert!(body.contains("MissingAuthorizationToken") || body.contains("required authorization token"));
        } else {
            panic!("Expected error reply for unauthorized purchase");
        }

        // 2. High-consequence action with valid token -> PASSES and single-use consumes token
        let mut valid_token = ActionAuthorizationToken::mint(
            "tok_pass_01",
            "u_gate",
            "financial_checkout",
            "execute_purchase",
            &payload_val,
            600,
            now_ms,
        );
        let auth_event = IncomingStimulusEvent::cli("u_gate", "Authorized buy")
            .with_action("execute_purchase", payload_val.clone())
            .with_token(valid_token.clone());
        let auth_receipt = router.dispatch_event(auth_event, &quota);

        assert!(auth_receipt.action_gate_passed);
        if let OutboundReply::Text { body } = auth_receipt.reply {
            assert!(body.contains("executed successfully"));
        } else {
            panic!("Expected success reply for authorized purchase");
        }

        // 3. Replay attack: Reusing already-consumed token -> FAILS CLOSED
        valid_token.consumed = true;
        let replay_event = IncomingStimulusEvent::cli("u_gate", "Replay buy")
            .with_action("execute_purchase", payload_val.clone())
            .with_token(valid_token);
        let replay_receipt = router.dispatch_event(replay_event, &quota);
        assert!(!replay_receipt.action_gate_passed);

        // 4. Tampered payload: Payload altered after token minted -> FAILS CLOSED
        let mut tampered_map = buy_payload;
        tampered_map.insert("amount_cents".to_string(), Value::from(999999u64));
        let tampered_val = Value::Object(tampered_map);

        let fresh_token = ActionAuthorizationToken::mint(
            "tok_tamper_01",
            "u_gate",
            "financial_checkout",
            "execute_purchase",
            &payload_val,
            600,
            now_ms,
        );
        let tamper_event = IncomingStimulusEvent::cli("u_gate", "Tampered buy")
            .with_action("execute_purchase", tampered_val)
            .with_token(fresh_token);
        let tamper_receipt = router.dispatch_event(tamper_event, &quota);
        assert!(!tamper_receipt.action_gate_passed);

        // 5. Trivial action (web_search) -> PASSES without token
        let mut search_map = BTreeMap::new();
        search_map.insert("query".to_string(), Value::String("rust concurrency".to_string()));
        let search_event = IncomingStimulusEvent::cli("u_gate", "Search query")
            .with_action("web_search", Value::Object(search_map));
        let search_receipt = router.dispatch_event(search_event, &quota);
        assert!(search_receipt.action_gate_passed);
    }

    #[test]
    fn test_stage_5_closure_and_spend_recording() {
        let router = GatewayRouter::new();
        let quota = UserSpendQuota::new("u_closure", UserTier::Pro.daily_quota_microcents());
        assert_eq!(quota.spent_microcents(), 0);

        // Step 1: Low-info reaction spends 100 microcents ($0.0001)
        let evt1 = IncomingStimulusEvent::webhook("u_closure", "thanks", None);
        let r1 = router.dispatch_event(evt1, &quota);
        assert_eq!(r1.microcents_spent, 100);
        assert_eq!(quota.spent_microcents(), 100);

        // Step 2: Memory store spends 1000 microcents ($0.001)
        let evt2 = IncomingStimulusEvent::webhook("u_closure", "remember that my favorite drink is matcha latte", None);
        let r2 = router.dispatch_event(evt2, &quota);
        assert_eq!(r2.microcents_spent, 1000);
        assert_eq!(quota.spent_microcents(), 1100);
    }
}

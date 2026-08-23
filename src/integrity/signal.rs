//! Signal layer (10XE C1/C4, D1): low-confidence patterns pause the run and
//! surface a gap to a human instead of guessing.

use crate::provider::ToolCallReq;

pub const HEDGES: [&str; 5] = [
    "i'm not sure",
    "i cannot determine",
    "i am unable to",
    "i don't have enough information",
    "it seems impossible",
];

pub struct SignalState {
    last_sig: Option<String>,
    fail_streak: u32,
}

impl SignalState {
    pub fn new() -> Self {
        SignalState { last_sig: None, fail_streak: 0 }
    }

    /// Feed one executed tool call + its (quarantine-wrapped) observation.
    /// Returns Some(reason) when a human should be pulled in.
    pub fn observe(&mut self, call: &ToolCallReq, wrapped_out: &str) -> Option<String> {
        let failed = wrapped_out.contains("exit: ")
            && !wrapped_out.contains("\nexit: 0\n")
            && !wrapped_out.starts_with("<tool_output tool=\"bash\" trust=\"untrusted\">\nexit: 0")
            || wrapped_out.contains("VALIDATION FAILED")
            || wrapped_out.contains("timed out");

        let sig = format!("{}:{}", call.name, {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut h = DefaultHasher::new();
            call.arguments.hash(&mut h);
            h.finish()
        });

        if self.last_sig.as_deref() == Some(sig.as_str()) && failed {
            self.fail_streak += 1;
        } else {
            self.fail_streak = if failed { 1 } else { 0 };
        }
        self.last_sig = Some(sig);

        if self.fail_streak >= 3 {
            return Some(format!(
                "same tool call failed {} times in a row ({})",
                self.fail_streak, call.name
            ));
        }
        None
    }

    pub fn check_assistant(content: &str) -> Option<String> {
        let low = content.to_lowercase();
        for h in HEDGES {
            if low.contains(h) {
                return Some(format!("model expressed low confidence: \"{h}\""));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(args: &str) -> ToolCallReq {
        ToolCallReq { id: "t".into(), name: "bash".into(), arguments: args.into(), extra: None }
    }

    #[test]
    fn triple_identical_failure_escalates() {
        let mut s = SignalState::new();
        let c = call("{\"command\":\"badcmd\"}");
        assert!(s.observe(&c, "<tool_output tool=\"bash\">\nexit: 127\n</tool_output>").is_none());
        assert!(s.observe(&c, "<tool_output tool=\"bash\">\nexit: 127\n</tool_output>").is_none());
        assert!(s.observe(&c, "<tool_output tool=\"bash\">\nexit: 127\n</tool_output>").is_some());
    }

    #[test]
    fn hedge_detection() {
        assert!(SignalState::check_assistant("I'm not sure how to proceed").is_some());
        assert!(SignalState::check_assistant("Done, file written.").is_none());
    }
}

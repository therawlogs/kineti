use regex::Regex;
use std::sync::OnceLock;

use crate::provider::ToolCallReq;

/// ASI-01 defense (D1): tool output is DATA, never instructions.
/// Instruction-shaped lines are flagged before the model ever sees them.
fn patterns() -> &'static Vec<Regex> {
    static P: OnceLock<Vec<Regex>> = OnceLock::new();
    P.get_or_init(|| {
        [
            r"(?i)ignore\s+(all\s+)?(previous|prior|above)\s+(instructions|prompts?)",
            r"(?i)disregard\s+(all\s+)?(previous|prior|above)",
            r"(?i)\[?\s*system\s+(note|prompt|message)\s*:?\s*\]?",
            r"(?i)you\s+are\s+now\s+(a|an|the)",
            r"(?i)new\s+instructions?\s*:",
            r"(?i)assistant\s*>",
            r"(?i)reveal|print|forward.{0,30}(api|key|token|secret|credential)",
        ]
        .iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect()
    })
}

/// Cheap structural pre-check on tool arguments before execution.
#[allow(dead_code)] // structural pre-check hook, v0.2 arg schema enforcement
pub fn check_args(call: &ToolCallReq) -> Result<(), String> {
    serde_json::from_str::<serde_json::Value>(&call.arguments)
        .map_err(|e| format!("arguments are not valid JSON ({e})"))?;
    Ok(())
}

pub fn wrap_output(tool: &str, out: String) -> String {
    let mut flagged = false;
    let mut sanitized_lines: Vec<String> = Vec::new();
    for line in out.lines() {
        let hit = patterns().iter().any(|re| re.is_match(line));
        if hit {
            flagged = true;
            sanitized_lines.push(format!("[QUARANTINED-INSTRUCTION-SHAPED-LINE] {}", line));
        } else {
            sanitized_lines.push(line.to_string());
        }
    }
    let body = sanitized_lines.join("\n");
    format!(
        "<tool_output tool=\"{tool}\" trust=\"untrusted\"{}>\n{}\n</tool_output>",
        if flagged { " quarantined=\"true\"" } else { "" },
        body
    )
}

pub fn last_was_flagged(output: &str) -> bool {
    output.contains("quarantined=\"true\"")
}

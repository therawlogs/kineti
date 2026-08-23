//! Authority tiers (ASI-03, D1): tool allowlists per stage band. Stages 1–6
//! are read-only (no code may exist before spec approval — ETHOS §10.2);
//! unauthorized use escalates to a human prompt.

pub fn allowed_for_stage(stage: u8) -> &'static [&'static str] {
    if stage <= 6 {
        &["read_file", "glob", "grep"]
    } else {
        // full suite once the human has approved the spec
        &["read_file", "glob", "grep", "write_file", "edit_file", "bash"]
    }
}

pub fn authorize_interactive(tool: &str, stage: u8) -> Result<(), String> {
    if allowed_for_stage(stage).contains(&tool) {
        return Ok(());
    }
    // escalation = human decision (never silent)
    eprint!(
        "⛔ ESCALATION: '{tool}' is outside tier for stage {stage}. Allow just this once? [y/N] "
    );
    let mut line = String::new();
    if std::io::stdin().read_line(&mut line).is_ok() && line.trim().eq_ignore_ascii_case("y") {
        println!("   human granted one-shot authority for '{tool}'");
        Ok(())
    } else {
        Err(format!("human denied '{tool}' at stage {stage}"))
    }
}

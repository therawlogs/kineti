//! Clean-files scanner (ETHOS §8.2): committed files contain no personal
//! names, no home paths, no secrets. Zero matches required before ship.
//! Emails are deliberately NOT flagged — licenses/credits carry them legally.

use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

/// Scan output must not echo personal names either — mask /Users|x/home tails.
fn redact_home(s: &str) -> String {
    static HOME: OnceLock<Regex> = OnceLock::new();
    let re = HOME.get_or_init(|| Regex::new(r"/(Users|home)/[A-Za-z0-9_.\-]+").unwrap());
    re.replace_all(s, "/$1/[REDACTED-HOME]").to_string()
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub path: String,
    pub line: usize,
    pub kind: String,
    /// matched line, truncated and secret-redacted before display
    pub snippet: String,
}

fn default_defs() -> [(&'static str, &'static str); 7] {
    [
        ("home-path", r"/Users/[A-Za-z0-9_.\-]+"),
        ("home-path", r"/home/[A-Za-z0-9_.\-]+"),
        ("secret", r"xai-[A-Za-z0-9]{20,}"),
        ("secret", r"AIza[0-9A-Za-z_\-]{30,}"),
        ("secret", r"sk-[A-Za-z0-9_\-]{20,}"),
        ("secret", r"gh[pousr]_[A-Za-z0-9_]{20,}"),
        ("secret", r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
    ]
}

/// Built-in forbidden shapes plus caller-supplied case-insensitive terms
/// (team/client names from `[clean_files] forbid` in kineti.toml).
pub fn patterns(extra: &[String]) -> Vec<(String, Regex)> {
    let mut out: Vec<(String, Regex)> = default_defs()
        .iter()
        .filter_map(|(k, p)| Regex::new(p).ok().map(|re| ((*k).to_string(), re)))
        .collect();
    for term in extra {
        let t = term.trim();
        if t.is_empty() {
            continue;
        }
        if let Ok(re) = Regex::new(&format!("(?i){}", regex::escape(t))) {
            out.push(("forbidden-term".to_string(), re));
        }
    }
    out
}

/// Walk the project (skipping .git/target/.kineti/node_modules like evidence
/// fingerprints do) and return every forbidden match. Empty result = clean.
/// A line ending in `kineti-clean-ignore` opts itself out — used ONLY by
/// scanner test fixtures; every use is auditable in review.
pub fn scan(root: &Path, extra: &[String]) -> Vec<Finding> {
    let mut files = vec![];
    crate::tools::walk_all(root, &mut files);
    files.sort();
    let pats = patterns(extra);

    let mut out = Vec::new();
    for f in files {
        let Ok(content) = std::fs::read_to_string(&f) else {
            continue; // binary or unreadable — not scannable text
        };
        let rel = f.strip_prefix(root).unwrap_or(&f).to_string_lossy().to_string();
        for (li, line) in content.lines().enumerate() {
            if line.contains("kineti-clean-ignore") {
                continue;
            }
            for (kind, re) in &pats {
                if re.is_match(line) {
                    let snippet: String = line.trim().chars().take(80).collect();
                    out.push(Finding {
                        path: rel.clone(),
                        line: li + 1,
                        kind: kind.clone(),
                        snippet: redact_home(&super::egress::redact(&snippet)),
                    });
                }
            }
        }
    }
    out
}

/// Ship-gate form of the scan (ETHOS §8.2): zero matches or refuse.
pub fn gate(root: &Path, extra: &[String]) -> Result<(), String> {
    let findings = scan(root, extra);
    if findings.is_empty() {
        return Ok(());
    }
    let preview = findings
        .iter()
        .take(5)
        .map(|f| format!("{}:{} [{}]", f.path, f.line, f.kind))
        .collect::<Vec<_>>()
        .join("; ");
    Err(format!(
        "SHIP REFUSED — clean-files scan found {} violation(s): {}",
        findings.len(),
        preview
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_home_path_and_secret_shapes() {
        let d = std::env::temp_dir().join(format!("kcf-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&d);
        std::fs::write(d.join("a.txt"), "see /Users/someone/file").unwrap(); // kineti-clean-ignore (fixture)
        let found = scan(&d, &[]);
        assert!(found.iter().any(|f| f.kind == "home-path"));
        assert!(found.iter().all(|f| !f.snippet.contains("someone")));
    }
}

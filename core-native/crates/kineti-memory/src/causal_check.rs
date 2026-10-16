//! Small evidence-backed incident note prototype.
//!
//! This module records a bounded incident description (why/how/who/what/where)
//! with explicit source IDs. It is a chronological append-only journal, not an
//! automated causal inference engine. Corrections supersede prior entries in
//! the current report while preserving the earlier record for inspection.

use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// One sourced observation or correction in an incident history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncidentEntry {
    /// Stable caller-provided entry identity.
    pub id: String,
    /// Unix timestamp in milliseconds, supplied by the caller.
    pub at_ms: u64,
    /// Actor identified by the source evidence.
    pub who: String,
    /// Action or mechanism identified by the source evidence.
    pub how: String,
    /// Observed outcome or failure.
    pub what: String,
    /// Affected component or location.
    pub where_: String,
    /// Explanation supported by the cited source.
    pub why: String,
    /// IDs for logs, test results, or other inspectable evidence.
    pub sources: Vec<String>,
    /// Entry this one corrects, if any.
    pub corrects: Option<String>,
}

/// Deterministic current view of one incident journal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncidentReport {
    /// Active entries in chronological order.
    pub entries: Vec<IncidentEntry>,
}

impl IncidentReport {
    /// Renders stable plain text with each assertion's evidence IDs.
    pub fn render(&self) -> String {
        let mut out = String::from("Causal Check prototype report\n");
        for entry in &self.entries {
            out.push_str(&format!(
                "[{} @ {}] why: {}; how: {}; who: {}; what: {}; where: {}; sources: {}\n",
                entry.id,
                entry.at_ms,
                entry.why,
                entry.how,
                entry.who,
                entry.what,
                entry.where_,
                entry.sources.join(",")
            ));
        }
        out
    }
}

/// Append-only file journal for one bounded incident.
#[derive(Debug)]
pub struct IncidentJournal {
    path: PathBuf,
    entries: Vec<IncidentEntry>,
}

impl IncidentJournal {
    /// Opens or creates a journal and restores its chronological history.
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        if !path.exists() {
            File::create(&path)?.sync_all()?;
        }
        let mut entries = Vec::new();
        for line in BufReader::new(File::open(&path)?).lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            entries.push(decode_entry(&line)?);
        }
        Ok(Self { path, entries })
    }

    /// Appends a sourced entry, rejecting duplicate IDs and empty evidence.
    pub fn append(&mut self, entry: IncidentEntry) -> std::io::Result<()> {
        if entry.id.trim().is_empty()
            || entry.sources.is_empty()
            || entry.sources.iter().any(|s| s.trim().is_empty())
            || self.entries.iter().any(|old| old.id == entry.id)
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "entry needs a unique ID and non-empty source IDs",
            ));
        }
        if let Some(target) = &entry.corrects {
            if !self.entries.iter().any(|old| &old.id == target) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "correction target does not exist",
                ));
            }
        }
        let mut file = OpenOptions::new().append(true).open(&self.path)?;
        writeln!(file, "{}", encode_entry(&entry))?;
        file.sync_all()?;
        self.entries.push(entry);
        Ok(())
    }

    /// Returns current active entries; corrected entries remain in the file but are hidden.
    pub fn report(&self) -> IncidentReport {
        let superseded: HashSet<&str> = self
            .entries
            .iter()
            .filter_map(|entry| entry.corrects.as_deref())
            .collect();
        let mut entries: Vec<_> = self
            .entries
            .iter()
            .filter(|entry| !superseded.contains(entry.id.as_str()))
            .cloned()
            .collect();
        entries.sort_by(|a, b| a.at_ms.cmp(&b.at_ms).then_with(|| a.id.cmp(&b.id)));
        IncidentReport { entries }
    }

    /// Returns all persisted entries, including corrected history.
    pub fn history(&self) -> &[IncidentEntry] {
        &self.entries
    }
}

fn encode_entry(e: &IncidentEntry) -> String {
    let fields = [
        e.id.clone(),
        e.at_ms.to_string(),
        e.who.clone(),
        e.how.clone(),
        e.what.clone(),
        e.where_.clone(),
        e.why.clone(),
        e.sources.join("\u{1f}"),
        e.corrects.clone().unwrap_or_default(),
    ];
    format!(
        "v1\t{}",
        fields
            .iter()
            .map(|s| encode(s))
            .collect::<Vec<_>>()
            .join("\t")
    )
}

fn decode_entry(line: &str) -> std::io::Result<IncidentEntry> {
    let mut fields = line.split('\t');
    if fields.next() != Some("v1") {
        return invalid("unknown journal version");
    }
    let decoded: Result<Vec<String>, _> = fields.map(decode).collect();
    let decoded = decoded?;
    if decoded.len() != 9 {
        return invalid("malformed journal entry");
    }
    let at_ms = decoded[1]
        .parse()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid timestamp"))?;
    Ok(IncidentEntry {
        id: decoded[0].clone(),
        at_ms,
        who: decoded[2].clone(),
        how: decoded[3].clone(),
        what: decoded[4].clone(),
        where_: decoded[5].clone(),
        why: decoded[6].clone(),
        sources: if decoded[7].is_empty() {
            vec![]
        } else {
            decoded[7].split('\u{1f}').map(str::to_owned).collect()
        },
        corrects: if decoded[8].is_empty() {
            None
        } else {
            Some(decoded[8].clone())
        },
    })
}

fn encode(s: &str) -> String {
    let mut out = String::new();
    for byte in s.as_bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.~".contains(byte) {
            out.push(*byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn decode(s: &str) -> std::io::Result<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len() {
                return invalid("truncated escape");
            }
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid escape")
            })?;
            out.push(u8::from_str_radix(hex, 16).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid escape")
            })?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid UTF-8"))
}

fn invalid<T>(message: &str) -> std::io::Result<T> {
    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        message,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, at_ms: u64, why: &str, corrects: Option<&str>) -> IncidentEntry {
        IncidentEntry {
            id: id.into(),
            at_ms,
            who: "release-agent".into(),
            how: "published stale lockfile".into(),
            what: "release check failed".into(),
            where_: "package pipeline".into(),
            why: why.into(),
            sources: vec!["test-log:case-17".into(), "receipt:abc123".into()],
            corrects: corrects.map(str::to_owned),
        }
    }

    #[test]
    fn deterministic_report_retrieval_correction_and_restart() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/causal-check-test.journal");
        let _ = fs::remove_file(&path);
        let mut journal = IncidentJournal::open(&path).unwrap();
        journal
            .append(entry(
                "e1",
                100,
                "dependency lock declaration did not match resolved package",
                None,
            ))
            .unwrap();
        let initial = journal.report().render();
        assert_eq!(initial, journal.report().render());
        assert!(initial.contains("why: dependency lock declaration did not match resolved package"));
        assert!(initial.contains("sources: test-log:case-17,receipt:abc123"));
        journal
            .append(entry(
                "e2",
                200,
                "the failure was a registry timeout",
                Some("e1"),
            ))
            .unwrap();

        let reopened = IncidentJournal::open(&path).unwrap();
        assert_eq!(reopened.history().len(), 2);
        let report = reopened.report();
        assert_eq!(report.entries.len(), 1);
        assert_eq!(report.entries[0].id, "e2");
        assert!(!report.render().contains("did not match resolved package"));
        assert!(report.render().contains("registry timeout"));
        assert_eq!(report.render(), reopened.report().render());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn rejects_unsourced_entries_and_unknown_correction_targets() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/causal-check-invalid.journal");
        let _ = fs::remove_file(&path);
        let mut journal = IncidentJournal::open(&path).unwrap();
        let mut unsourced = entry("x", 1, "unsupported claim", None);
        unsourced.sources.clear();
        assert!(journal.append(unsourced).is_err());
        assert!(journal
            .append(entry("y", 2, "correction", Some("missing")))
            .is_err());
        let _ = fs::remove_file(path);
    }
}

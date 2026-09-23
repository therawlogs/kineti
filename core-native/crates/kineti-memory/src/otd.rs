//! # Runtime Ontology Trigger Data (OTD) Engine (`otd`)
//!
//! Binds tenant-specific payloads to the Universal 20-Entity Provenance Kernel
//! at runtime via dynamic schemas with JMESPath-lite extraction paths.
//!
//! 100% Safe Rust, zero external crates. The path language supports dotted
//! keys plus `[index]` steps (for example `ticket.message`, `items[0].id`).
//! Rejected paths, unknown entity names, and missing required fields fail
//! closed with a typed error instead of silently binding nothing.

use kineti_core::kernel::EntityType;
use std::collections::HashMap;

/// A single field binding from a payload path to a kernel entity type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtdBinding {
    /// Kernel entity name (for example `intent`, `tool_call`). Case-insensitive.
    pub entity: String,
    /// JMESPath-lite extraction path (for example `ticket.message`).
    pub path: String,
    /// When true, a missing path fails the whole apply instead of skipping.
    pub required: bool,
}

/// A named, versioned set of bindings applied together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtdSchema {
    /// Schema name (for example `support_ticket_v1`).
    pub name: String,
    /// Schema version, bumped on every binding change.
    pub version: u32,
    /// Bindings applied in order.
    pub bindings: Vec<OtdBinding>,
}

/// A successfully extracted fact bound to a kernel entity.
#[derive(Debug, Clone, PartialEq)]
pub struct BoundFact {
    /// Canonical kernel entity name.
    pub entity: String,
    /// Extraction path that produced this fact.
    pub path: String,
    /// Extracted scalar value as text.
    pub value: String,
}

/// Typed errors for schema validation and payload extraction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OtdError {
    /// Schema has no bindings.
    EmptySchema,
    /// Entity name is not one of the 20 kernel types.
    UnknownEntity(String),
    /// Extraction path is empty or malformed.
    BadPath(String),
    /// Two bindings write the same entity from the same path.
    DuplicateBinding(String),
    /// Required path missing from payload.
    MissingRequired(String),
    /// Payload is not valid lite-JSON.
    BadPayload(String),
}

impl std::fmt::Display for OtdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySchema => write!(f, "OTD schema has no bindings"),
            Self::UnknownEntity(e) => write!(f, "OTD unknown kernel entity: {e}"),
            Self::BadPath(p) => write!(f, "OTD malformed path: {p}"),
            Self::DuplicateBinding(k) => write!(f, "OTD duplicate binding: {k}"),
            Self::MissingRequired(p) => write!(f, "OTD missing required path: {p}"),
            Self::BadPayload(e) => write!(f, "OTD bad payload: {e}"),
        }
    }
}

impl std::error::Error for OtdError {}

/// Minimal JSON value model for tenant payloads (objects, arrays, scalars).
#[derive(Debug, Clone, PartialEq)]
pub enum OtdValue {
    /// Null literal.
    Null,
    /// Boolean literal.
    Bool(bool),
    /// Number literal, kept as text to avoid float parsing.
    Num(String),
    /// String literal.
    Str(String),
    /// Ordered array.
    Arr(Vec<OtdValue>),
    /// String-keyed object.
    Map(HashMap<String, OtdValue>),
}

fn skip_ws(s: &[u8], pos: &mut usize) {
    while *pos < s.len() && (s[*pos] == b' ' || s[*pos] == b'\t' || s[*pos] == b'\n' || s[*pos] == b'\r') {
        *pos += 1;
    }
}

fn parse_string(s: &[u8], pos: &mut usize) -> Result<String, String> {
    if *pos >= s.len() || s[*pos] != b'"' {
        return Err("expected string".to_string());
    }
    *pos += 1;
    let mut out = String::new();
    while *pos < s.len() {
        match s[*pos] {
            b'"' => {
                *pos += 1;
                return Ok(out);
            }
            b'\\' => {
                *pos += 1;
                if *pos >= s.len() {
                    return Err("dangling escape".to_string());
                }
                match s[*pos] {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'n' => out.push('\n'),
                    b't' => out.push('\t'),
                    b'r' => out.push('\r'),
                    other => {
                        out.push('\\');
                        out.push(other as char);
                    }
                }
                *pos += 1;
            }
            c => {
                out.push(c as char);
                *pos += 1;
            }
        }
    }
    Err("unterminated string".to_string())
}

fn parse_value(s: &[u8], pos: &mut usize) -> Result<OtdValue, String> {
    skip_ws(s, pos);
    if *pos >= s.len() {
        return Err("unexpected end".to_string());
    }
    match s[*pos] {
        b'"' => Ok(OtdValue::Str(parse_string(s, pos)?)),
        b'{' => parse_object(s, pos),
        b'[' => parse_array(s, pos),
        b't' if s[*pos..].starts_with(b"true") => {
            *pos += 4;
            Ok(OtdValue::Bool(true))
        }
        b'f' if s[*pos..].starts_with(b"false") => {
            *pos += 5;
            Ok(OtdValue::Bool(false))
        }
        b'n' if s[*pos..].starts_with(b"null") => {
            *pos += 4;
            Ok(OtdValue::Null)
        }
        c if c == b'-' || c.is_ascii_digit() => {
            let start = *pos;
            while *pos < s.len() && (s[*pos].is_ascii_digit() || matches!(s[*pos], b'-' | b'+' | b'.' | b'e' | b'E')) {
                *pos += 1;
            }
            Ok(OtdValue::Num(String::from_utf8_lossy(&s[start..*pos]).into_owned()))
        }
        other => Err(format!("unexpected byte: {other}")),
    }
}

fn parse_object(s: &[u8], pos: &mut usize) -> Result<OtdValue, String> {
    *pos += 1; // consume {
    let mut map = HashMap::new();
    skip_ws(s, pos);
    if *pos < s.len() && s[*pos] == b'}' {
        *pos += 1;
        return Ok(OtdValue::Map(map));
    }
    loop {
        skip_ws(s, pos);
        let key = parse_string(s, pos)?;
        skip_ws(s, pos);
        if *pos >= s.len() || s[*pos] != b':' {
            return Err("expected colon".to_string());
        }
        *pos += 1;
        let val = parse_value(s, pos)?;
        map.insert(key, val);
        skip_ws(s, pos);
        if *pos >= s.len() {
            return Err("unterminated object".to_string());
        }
        match s[*pos] {
            b',' => {
                *pos += 1;
            }
            b'}' => {
                *pos += 1;
                return Ok(OtdValue::Map(map));
            }
            _ => return Err("expected comma or brace".to_string()),
        }
    }
}

fn parse_array(s: &[u8], pos: &mut usize) -> Result<OtdValue, String> {
    *pos += 1; // consume [
    let mut items = Vec::new();
    skip_ws(s, pos);
    if *pos < s.len() && s[*pos] == b']' {
        *pos += 1;
        return Ok(OtdValue::Arr(items));
    }
    loop {
        let val = parse_value(s, pos)?;
        items.push(val);
        skip_ws(s, pos);
        if *pos >= s.len() {
            return Err("unterminated array".to_string());
        }
        match s[*pos] {
            b',' => {
                *pos += 1;
            }
            b']' => {
                *pos += 1;
                return Ok(OtdValue::Arr(items));
            }
            _ => return Err("expected comma or bracket".to_string()),
        }
    }
}

/// Parses a lite-JSON document into an [`OtdValue`].
pub fn parse_payload(text: &str) -> Result<OtdValue, OtdError> {
    let bytes = text.as_bytes();
    let mut pos = 0;
    let val = parse_value(bytes, &mut pos).map_err(OtdError::BadPayload)?;
    skip_ws(bytes, &mut pos);
    if pos != bytes.len() {
        return Err(OtdError::BadPayload("trailing bytes".to_string()));
    }
    Ok(val)
}

/// Extracts a scalar value at a dotted path with optional `[index]` steps.
/// Returns None when any step is missing or lands on a non-scalar.
pub fn extract_path(root: &OtdValue, path: &str) -> Option<String> {
    if path.is_empty() {
        return None;
    }
    let mut current = root;
    for segment in path.split('.') {
        if segment.is_empty() {
            return None;
        }
        // Split `key[0][1]` into key plus indices.
        let (key, rest) = match segment.find('[') {
            Some(i) => (&segment[..i], &segment[i..]),
            None => (segment, ""),
        };
        if !key.is_empty() {
            match current {
                OtdValue::Map(m) => current = m.get(key)?,
                _ => return None,
            }
        }
        let mut tail = rest;
        while let Some(open) = tail.find('[') {
            let close = tail.find(']')?;
            let idx: usize = tail[open + 1..close].parse().ok()?;
            match current {
                OtdValue::Arr(items) => current = items.get(idx)?,
                _ => return None,
            }
            tail = &tail[close + 1..];
        }
        if !tail.is_empty() {
            return None;
        }
    }
    match current {
        OtdValue::Str(s) => Some(s.clone()),
        OtdValue::Num(n) => Some(n.clone()),
        OtdValue::Bool(b) => Some(b.to_string()),
        OtdValue::Null | OtdValue::Arr(_) | OtdValue::Map(_) => None,
    }
}

impl OtdSchema {
    /// Validates entity names, paths, and duplicates. Fails closed.
    pub fn validate(&self) -> Result<(), OtdError> {
        if self.bindings.is_empty() {
            return Err(OtdError::EmptySchema);
        }
        let mut seen = std::collections::HashSet::new();
        for b in &self.bindings {
            if EntityType::from_str(&b.entity).is_none() {
                return Err(OtdError::UnknownEntity(b.entity.clone()));
            }
            if b.path.is_empty() || b.path.split('.').any(str::is_empty) {
                return Err(OtdError::BadPath(b.path.clone()));
            }
            let key = format!("{}\u{0}{}", b.entity.to_ascii_lowercase(), b.path);
            if !seen.insert(key.clone()) {
                return Err(OtdError::DuplicateBinding(key));
            }
        }
        Ok(())
    }

    /// Applies the schema to a lite-JSON payload, returning bound kernel facts.
    pub fn apply(&self, payload: &str) -> Result<Vec<BoundFact>, OtdError> {
        self.validate()?;
        let root = parse_payload(payload)?;
        let mut out = Vec::new();
        for b in &self.bindings {
            match extract_path(&root, &b.path) {
                Some(value) => {
                    let entity = EntityType::from_str(&b.entity)
                        .map(|e| e.as_str().to_string())
                        .unwrap_or_else(|| b.entity.clone());
                    out.push(BoundFact { entity, path: b.path.clone(), value });
                }
                None if b.required => return Err(OtdError::MissingRequired(b.path.clone())),
                None => {}
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ticket_schema() -> OtdSchema {
        OtdSchema {
            name: "support_ticket_v1".to_string(),
            version: 1,
            bindings: vec![
                OtdBinding { entity: "intent".to_string(), path: "ticket.message".to_string(), required: true },
                OtdBinding { entity: "goal".to_string(), path: "ticket.priority".to_string(), required: false },
                OtdBinding { entity: "actor".to_string(), path: "items[0].id".to_string(), required: false },
            ],
        }
    }

    #[test]
    fn test_schema_validates_ok() {
        assert!(ticket_schema().validate().is_ok());
    }

    #[test]
    fn test_schema_rejects_unknown_entity() {
        let s = OtdSchema {
            name: "bad".to_string(),
            version: 1,
            bindings: vec![OtdBinding { entity: "spaceship".to_string(), path: "a.b".to_string(), required: false }],
        };
        assert_eq!(s.validate(), Err(OtdError::UnknownEntity("spaceship".to_string())));
    }

    #[test]
    fn test_schema_rejects_bad_path_and_duplicates() {
        let bad = OtdSchema {
            name: "bad".to_string(),
            version: 1,
            bindings: vec![OtdBinding { entity: "goal".to_string(), path: "a..b".to_string(), required: false }],
        };
        assert!(matches!(bad.validate(), Err(OtdError::BadPath(_))));
        let dup = OtdSchema {
            name: "dup".to_string(),
            version: 1,
            bindings: vec![
                OtdBinding { entity: "goal".to_string(), path: "a.b".to_string(), required: false },
                OtdBinding { entity: "GOAL".to_string(), path: "a.b".to_string(), required: false },
            ],
        };
        assert!(matches!(dup.validate(), Err(OtdError::DuplicateBinding(_))));
    }

    #[test]
    fn test_extract_nested_and_index_paths() {
        let root = parse_payload(r#"{"ticket":{"message":"refund please","tags":["billing","urgent"]},"items":[{"id":"u1"}]}"#).unwrap();
        assert_eq!(extract_path(&root, "ticket.message"), Some("refund please".to_string()));
        assert_eq!(extract_path(&root, "ticket.tags[1]"), Some("urgent".to_string()));
        assert_eq!(extract_path(&root, "items[0].id"), Some("u1".to_string()));
        assert_eq!(extract_path(&root, "ticket.missing"), None);
        assert_eq!(extract_path(&root, "ticket.tags[9]"), None);
        assert_eq!(extract_path(&root, "ticket"), None);
    }

    #[test]
    fn test_apply_binds_and_skips_optional() {
        let facts = ticket_schema()
            .apply(r#"{"ticket":{"message":"hi","priority":"high"},"items":[{"id":"u1"}]}"#)
            .unwrap();
        assert_eq!(facts.len(), 3);
        assert_eq!(facts[0].entity, "Intent");
        assert_eq!(facts[0].value, "hi");
        let facts2 = ticket_schema()
            .apply(r#"{"ticket":{"message":"hi"},"items":[]}"#)
            .unwrap();
        assert_eq!(facts2.len(), 1);
    }

    #[test]
    fn test_apply_fails_closed_on_missing_required() {
        let err = ticket_schema().apply(r#"{"ticket":{}}"#).unwrap_err();
        assert_eq!(err, OtdError::MissingRequired("ticket.message".to_string()));
    }

    #[test]
    fn test_all_twenty_kernel_entities_accepted() {
        let bindings = EntityType::ALL
            .iter()
            .map(|e| OtdBinding { entity: e.as_str().to_string(), path: "v".to_string(), required: false });
        let s = OtdSchema { name: "all".to_string(), version: 1, bindings: bindings.collect() };
        assert!(s.validate().is_ok());
        let facts = s.apply(r#"{"v":"x"}"#).unwrap();
        assert_eq!(facts.len(), 20);
    }
}

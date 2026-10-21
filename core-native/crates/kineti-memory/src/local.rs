//! Current local OS-user/project memory. The encrypted TypeScript store is the
//! persistence authority; native graph/vector indexes are rebuilt from its live
//! view on every request. Legacy prototype engines are not an authorization API.
use crate::graph::UserPropertyGraph;
use crate::tombstone::TombstoneMask;
use crate::vector::{SearchMatch, VectorIndex};
use kineti_core::conversation::UserFact;
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// One current encrypted record after authorized local decryption.
#[derive(Clone, Debug)]
pub struct LocalMemoryRecord {
    /// Stable versioned memory record id.
    pub id: String,
    /// Fact category.
    pub category: String,
    /// Fact key.
    pub key: String,
    /// Current fact value; never a deleted or superseded version.
    pub value: String,
    /// Finite embedding values.
    pub embedding: Vec<f32>,
    /// Persisted creation time used to preserve native content identity.
    pub created_at_ms: u64,
}
/// A local current-memory session with no caller-selected user identity.
#[derive(Clone, Debug)]
pub struct PersistentLocalMemory {
    root: PathBuf,
    bun: PathBuf,
    bridge: PathBuf,
}
impl PersistentLocalMemory {
    /// Open a project using operator-selected installed Bun and bridge paths.
    /// The bridge derives identity from the actual OS UID and canonical project.
    pub fn open(root: &Path, bun: &Path, bridge: &Path) -> Result<Self, String> {
        let session = Self {
            root: root.canonicalize().map_err(|e| e.to_string())?,
            bun: bun.canonicalize().map_err(|e| e.to_string())?,
            bridge: bridge.canonicalize().map_err(|e| e.to_string())?,
        };
        session.current()?;
        Ok(session)
    }
    fn request(&self, request: Value) -> Result<Value, String> {
        let mut child = Command::new(&self.bun)
            .arg(&self.bridge)
            .current_dir(&self.root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        child
            .stdin
            .take()
            .ok_or("bridge stdin unavailable")?
            .write_all(&serde_json::to_vec(&request).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let result = child.wait_with_output().map_err(|e| e.to_string())?;
        if !result.status.success() {
            return Err(format!(
                "local memory refused: {}",
                String::from_utf8_lossy(&result.stderr)
            ));
        }
        if result.stdout.len() > 20 * 1024 * 1024 {
            return Err("native memory response too large".into());
        }
        let value: Value = serde_json::from_slice(&result.stdout).map_err(|e| e.to_string())?;
        if value["version"] != 1
            || !value["scope"]["user"]
                .as_str()
                .is_some_and(|s| s.starts_with("local-uid-"))
            || !value["scope"]["project"]
                .as_str()
                .is_some_and(|s| s.starts_with("local-project-v2-"))
        {
            return Err("invalid native bridge identity".into());
        }
        Ok(value)
    }
    fn current(&self) -> Result<(String, Vec<LocalMemoryRecord>), String> {
        let response = self.request(json!({"version":1,"operation":"snapshot"}))?;
        let namespace = format!(
            "{}:{}",
            response["scope"]["user"]
                .as_str()
                .ok_or("missing OS scope")?,
            response["scope"]["project"]
                .as_str()
                .ok_or("missing project scope")?
        );
        let values = response["records"]
            .as_array()
            .ok_or("missing current records")?;
        if values.len() > 1000 {
            return Err("native snapshot limit exceeded".into());
        }
        let mut records = Vec::new();
        for value in values {
            let text = |key: &str| {
                value[key]
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| format!("missing native field {key}"))
            };
            let embedding: Vec<f32> = value["embedding"]
                .as_array()
                .ok_or("missing embedding")?
                .iter()
                .map(|v| {
                    v.as_f64()
                        .map(|n| n as f32)
                        .filter(|n| n.is_finite())
                        .ok_or("invalid embedding")
                })
                .collect::<Result<_, _>>()?;
            if embedding.is_empty() || embedding.len() > 256 {
                return Err("invalid embedding dimensions".into());
            }
            records.push(LocalMemoryRecord {
                id: text("id")?,
                category: text("category")?,
                key: text("key")?,
                value: text("value")?,
                embedding,
                created_at_ms: value["created_at_ms"]
                    .as_u64()
                    .ok_or("invalid persisted time")?,
            });
        }
        Ok((namespace, records))
    }
    /// Persist a fact and its embedding through the encrypted authority.
    pub fn put(
        &self,
        id: &str,
        category: &str,
        key: &str,
        value: &str,
        embedding: &[f32],
    ) -> Result<(), String> {
        self.mutate("ingest", id, None, (category, key, value, embedding))
    }
    /// Append a correction; all retrieval immediately excludes the prior record.
    pub fn correct(
        &self,
        prior: &str,
        id: &str,
        category: &str,
        key: &str,
        value: &str,
        embedding: &[f32],
    ) -> Result<(), String> {
        self.mutate(
            "correct",
            id,
            Some(prior),
            (category, key, value, embedding),
        )
    }
    fn mutate(
        &self,
        operation: &str,
        id: &str,
        prior: Option<&str>,
        payload: (&str, &str, &str, &[f32]),
    ) -> Result<(), String> {
        let (category, key, value, embedding) = payload;
        if embedding.is_empty() || embedding.len() > 256 || embedding.iter().any(|n| !n.is_finite())
        {
            return Err("finite bounded embedding required".into());
        }
        self.request(json!({"version":1,"operation":operation,"id":id,"corrects":prior,"payload":{"category":category,"key":key,"value":value,"embedding":embedding}}))?;
        Ok(())
    }
    /// Delete the record and correction ancestry; only permitted audit metadata remains.
    pub fn delete(&self, id: &str) -> Result<(), String> {
        self.request(json!({"version":1,"operation":"delete","id":id,"confirm":id}))?;
        Ok(())
    }
    /// Rebuild the native graph from the current authorized project view.
    pub fn graph(&self, category: Option<&str>) -> Result<Vec<UserFact>, String> {
        let (namespace, records) = self.current()?;
        let graph = UserPropertyGraph::new();
        for record in records {
            graph.insert_fact(UserFact::new(
                &namespace,
                &record.category,
                &record.key,
                &record.value,
                1.0,
                Some(record.id),
                record.created_at_ms,
            ));
        }
        Ok(graph.query_active_facts(&namespace, category, &TombstoneMask::new()))
    }
    /// Search a freshly rebuilt, scoped native vector index. No historical index is reused.
    pub fn vectors(&self, query: &[f32], limit: usize) -> Result<Vec<SearchMatch>, String> {
        if query.is_empty()
            || query.len() > 256
            || query.iter().any(|n| !n.is_finite())
            || limit == 0
            || limit > 20
        {
            return Err("invalid native vector query".into());
        }
        let (namespace, records) = self.current()?;
        let index = VectorIndex::new();
        for record in records
            .into_iter()
            .filter(|r| r.embedding.len() == query.len())
        {
            index.insert_for_user(&namespace, record.id, record.embedding, record.value);
        }
        Ok(index.search_for_user(&namespace, query, limit, &TombstoneMask::new()))
    }
}

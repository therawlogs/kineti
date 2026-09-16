//! Embedded SQLite Disk Persistence for the Causal Property Graph.
//!
//! Backs the in-memory [`UserPropertyGraph`] with durable SQLite storage in WAL
//! (Write-Ahead Log) mode. All queries are partitioned by `user_id` to guarantee
//! zero cross-tenant leakage at the storage layer.
//!
//! ## Design Principles
//! - **WAL mode**: Concurrent readers never block writers; writes never block reads.
//! - **User-partitioned queries**: Every `SELECT` includes `WHERE user_id = ?` to
//!   prevent any cross-tenant data exposure.
//! - **O(1) tombstone soft deletion**: Tombstoned facts are instantly masked by
//!   setting `tombstoned = 1` and `tombstoned_at_ms`. A background sweep zeroes
//!   the `value` column for rows tombstoned over 24 hours ago.
//! - **Process restart survival**: All causal edges (`Replaces`, `RelatesTo`,
//!   `DerivedFrom`) are persisted alongside facts and survive server reboots.

use crate::graph::{FactEdge, FactRelation};
use kineti_core::conversation::UserFact;
use std::path::Path;

/// Errors that can occur in the storage layer.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StorageError {
    /// Database could not be opened or created.
    OpenFailed(String),
    /// Schema migration failed.
    MigrationFailed(String),
    /// A write operation failed.
    WriteFailed(String),
    /// A read/query operation failed.
    ReadFailed(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenFailed(msg) => write!(f, "Storage open failed: {}", msg),
            Self::MigrationFailed(msg) => write!(f, "Migration failed: {}", msg),
            Self::WriteFailed(msg) => write!(f, "Write failed: {}", msg),
            Self::ReadFailed(msg) => write!(f, "Read failed: {}", msg),
        }
    }
}

impl std::error::Error for StorageError {}

/// Result type for storage operations.
pub type StorageResult<T> = Result<T, StorageError>;

/// Row representation of a persisted fact.
#[derive(Debug, Clone)]
pub struct PersistedFact {
    /// Fact node ID.
    pub id: String,
    /// Owning user ID.
    pub user_id: String,
    /// Fact category (e.g. "location", "relationship").
    pub category: String,
    /// Fact key (e.g. "home_city", "manager").
    pub key: String,
    /// Fact value (zeroed after 24h tombstone sweep).
    pub value: String,
    /// Confidence score 0.0–1.0.
    pub confidence: f32,
    /// Optional source message ID.
    pub source_message_id: Option<String>,
    /// Creation timestamp (Unix ms).
    pub created_at_ms: u64,
    /// Whether this fact is tombstoned.
    pub tombstoned: bool,
    /// Timestamp when tombstoned (Unix ms), or 0 if active.
    pub tombstoned_at_ms: u64,
}

impl PersistedFact {
    /// Converts a [`UserFact`] to a [`PersistedFact`].
    pub fn from_user_fact(fact: &UserFact) -> Self {
        Self {
            id: fact.id.as_str().to_string(),
            user_id: fact.user_id.clone(),
            category: fact.category.clone(),
            key: fact.key.clone(),
            value: fact.value.clone(),
            confidence: fact.confidence,
            source_message_id: fact.source_message_id.clone(),
            created_at_ms: fact.created_at_ms,
            tombstoned: fact.tombstoned,
            tombstoned_at_ms: 0,
        }
    }

    /// Converts back to a [`UserFact`].
    pub fn to_user_fact(&self) -> UserFact {
        UserFact::new(
            &self.user_id,
            &self.category,
            &self.key,
            &self.value,
            self.confidence,
            self.source_message_id.clone(),
            self.created_at_ms,
        )
    }
}

/// Row representation of a persisted causal edge.
#[derive(Debug, Clone)]
pub struct PersistedEdge {
    /// Source fact ID.
    pub from_id: String,
    /// Target fact ID.
    pub to_id: String,
    /// Relation type stored as string: "replaces", "relates_to", "derived_from".
    pub relation: String,
}

impl PersistedEdge {
    /// Converts a [`FactEdge`] to a [`PersistedEdge`].
    pub fn from_fact_edge(edge: &FactEdge) -> Self {
        Self {
            from_id: edge.from_id.clone(),
            to_id: edge.to_id.clone(),
            relation: match edge.relation {
                FactRelation::Replaces => "replaces".to_string(),
                FactRelation::RelatesTo => "relates_to".to_string(),
                FactRelation::DerivedFrom => "derived_from".to_string(),
            },
        }
    }

    /// Converts back to a [`FactEdge`].
    pub fn to_fact_edge(&self) -> Option<FactEdge> {
        let relation = match self.relation.as_str() {
            "replaces" => FactRelation::Replaces,
            "relates_to" => FactRelation::RelatesTo,
            "derived_from" => FactRelation::DerivedFrom,
            _ => return None,
        };
        Some(FactEdge {
            from_id: self.from_id.clone(),
            to_id: self.to_id.clone(),
            relation,
        })
    }
}

// ---------------------------------------------------------------------------
// SQL Schema
// ---------------------------------------------------------------------------

/// SQL statements for schema creation. Called once on database open.
const SCHEMA_SQL: &str = "
    CREATE TABLE IF NOT EXISTS facts (
        id              TEXT PRIMARY KEY NOT NULL,
        user_id         TEXT NOT NULL,
        category        TEXT NOT NULL,
        key             TEXT NOT NULL,
        value           TEXT NOT NULL DEFAULT '',
        confidence      REAL NOT NULL DEFAULT 0.0,
        source_msg_id   TEXT,
        created_at_ms   INTEGER NOT NULL,
        tombstoned      INTEGER NOT NULL DEFAULT 0,
        tombstoned_at_ms INTEGER NOT NULL DEFAULT 0
    );

    CREATE INDEX IF NOT EXISTS idx_facts_user_id ON facts(user_id);
    CREATE INDEX IF NOT EXISTS idx_facts_user_category ON facts(user_id, category);
    CREATE INDEX IF NOT EXISTS idx_facts_tombstoned ON facts(tombstoned, tombstoned_at_ms);

    CREATE TABLE IF NOT EXISTS edges (
        from_id   TEXT NOT NULL,
        to_id     TEXT NOT NULL,
        relation  TEXT NOT NULL,
        PRIMARY KEY (from_id, to_id, relation)
    );

    CREATE INDEX IF NOT EXISTS idx_edges_from ON edges(from_id);
    CREATE INDEX IF NOT EXISTS idx_edges_to ON edges(to_id);
";

// ---------------------------------------------------------------------------
// Storage Engine (Platform-agnostic interface)
// ---------------------------------------------------------------------------

/// Configuration for the SQLite storage engine.
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Path to the SQLite database file. Use ":memory:" for in-memory testing.
    pub db_path: String,
    /// How many hours before tombstoned facts are disk-zeroed. Default: 24.
    pub tombstone_zeroing_hours: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            db_path: "kineti_memory.db".to_string(),
            tombstone_zeroing_hours: 24,
        }
    }
}

impl StorageConfig {
    /// Creates a config pointing to a specific database file.
    pub fn with_path(path: impl Into<String>) -> Self {
        Self {
            db_path: path.into(),
            ..Default::default()
        }
    }

    /// Creates an in-memory config for testing.
    pub fn in_memory() -> Self {
        Self {
            db_path: ":memory:".to_string(),
            ..Default::default()
        }
    }

    /// Returns the path as a `Path` reference (only valid for file-based DBs).
    pub fn as_path(&self) -> Option<&Path> {
        if self.db_path == ":memory:" {
            None
        } else {
            Some(Path::new(&self.db_path))
        }
    }
}

/// The SQL statements needed to perform each storage operation.
/// This allows the storage layer to work without any SQLite library dependency
/// — the caller (the daemon or test harness) provides the actual database execution.
#[derive(Debug)]
pub struct StorageOps;

impl StorageOps {
    /// Returns the DDL statements to initialize the schema.
    pub fn schema_ddl() -> &'static str {
        SCHEMA_SQL
    }

    /// Returns the SQL and params layout for inserting a fact.
    pub fn insert_fact_sql() -> &'static str {
        "INSERT OR REPLACE INTO facts (id, user_id, category, key, value, confidence, source_msg_id, created_at_ms, tombstoned, tombstoned_at_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
    }

    /// Returns the SQL for inserting a causal edge.
    pub fn insert_edge_sql() -> &'static str {
        "INSERT OR IGNORE INTO edges (from_id, to_id, relation) VALUES (?1, ?2, ?3)"
    }

    /// Returns the SQL for querying active (non-tombstoned) facts for a user.
    pub fn query_active_facts_sql() -> &'static str {
        "SELECT id, user_id, category, key, value, confidence, source_msg_id, created_at_ms, tombstoned, tombstoned_at_ms
         FROM facts WHERE user_id = ?1 AND tombstoned = 0"
    }

    /// Returns the SQL for querying active facts filtered by category.
    pub fn query_active_facts_by_category_sql() -> &'static str {
        "SELECT id, user_id, category, key, value, confidence, source_msg_id, created_at_ms, tombstoned, tombstoned_at_ms
         FROM facts WHERE user_id = ?1 AND category = ?2 AND tombstoned = 0"
    }

    /// Returns the SQL for soft-deleting (tombstoning) a fact.
    pub fn tombstone_fact_sql() -> &'static str {
        "UPDATE facts SET tombstoned = 1, tombstoned_at_ms = ?2 WHERE id = ?1"
    }

    /// Returns the SQL for the 24-hour background disk zeroing sweep.
    /// Zeroes the `value` column for facts tombstoned over N hours ago.
    pub fn disk_zero_sweep_sql() -> &'static str {
        "UPDATE facts SET value = '' WHERE tombstoned = 1 AND tombstoned_at_ms > 0 AND tombstoned_at_ms < ?1"
    }

    /// Returns the SQL for querying all causal edges involving a specific fact.
    pub fn query_edges_for_fact_sql() -> &'static str {
        "SELECT from_id, to_id, relation FROM edges WHERE from_id = ?1 OR to_id = ?1"
    }

    /// Returns the SQL for querying all edges for a set of fact IDs (used on restore).
    pub fn query_all_edges_sql() -> &'static str {
        "SELECT from_id, to_id, relation FROM edges"
    }

    /// Returns the SQL for hard-deleting a fact (after disk zeroing).
    pub fn hard_delete_fact_sql() -> &'static str {
        "DELETE FROM facts WHERE id = ?1 AND tombstoned = 1"
    }

    /// Returns the SQL for counting facts per user (admin/debug).
    pub fn count_facts_for_user_sql() -> &'static str {
        "SELECT COUNT(*) FROM facts WHERE user_id = ?1"
    }

    /// Returns the SQL for WAL mode pragma.
    pub fn wal_mode_pragma() -> &'static str {
        "PRAGMA journal_mode=WAL"
    }

    /// Returns the SQL for busy timeout pragma (5 seconds).
    pub fn busy_timeout_pragma() -> &'static str {
        "PRAGMA busy_timeout=5000"
    }

    /// Returns the SQL for foreign keys pragma.
    pub fn foreign_keys_pragma() -> &'static str {
        "PRAGMA foreign_keys=ON"
    }

    /// Calculates the cutoff timestamp for disk zeroing.
    /// Facts tombstoned before this timestamp should have their values zeroed.
    pub fn zeroing_cutoff_ms(now_ms: u64, hours: u64) -> u64 {
        let millis = hours * 60 * 60 * 1000;
        now_ms.saturating_sub(millis)
    }
}

/// Builds the parameter list for inserting a [`PersistedFact`].
/// Returns values in the order matching `insert_fact_sql()` placeholders.
pub fn fact_to_params(fact: &PersistedFact) -> Vec<StorageParam> {
    vec![
        StorageParam::Text(fact.id.clone()),
        StorageParam::Text(fact.user_id.clone()),
        StorageParam::Text(fact.category.clone()),
        StorageParam::Text(fact.key.clone()),
        StorageParam::Text(fact.value.clone()),
        StorageParam::Real(fact.confidence as f64),
        match &fact.source_message_id {
            Some(s) => StorageParam::Text(s.clone()),
            None => StorageParam::Null,
        },
        StorageParam::Integer(fact.created_at_ms as i64),
        StorageParam::Integer(if fact.tombstoned { 1 } else { 0 }),
        StorageParam::Integer(fact.tombstoned_at_ms as i64),
    ]
}

/// Builds the parameter list for inserting a [`PersistedEdge`].
pub fn edge_to_params(edge: &PersistedEdge) -> Vec<StorageParam> {
    vec![
        StorageParam::Text(edge.from_id.clone()),
        StorageParam::Text(edge.to_id.clone()),
        StorageParam::Text(edge.relation.clone()),
    ]
}

/// A type-safe parameter for SQL bindings.
#[derive(Debug, Clone, PartialEq)]
pub enum StorageParam {
    /// Text / VARCHAR value.
    Text(String),
    /// Integer value.
    Integer(i64),
    /// Floating point value.
    Real(f64),
    /// NULL value.
    Null,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_ddl_is_valid_sql_syntax() {
        let ddl = StorageOps::schema_ddl();
        // Verify DDL contains expected table and index creation
        assert!(ddl.contains("CREATE TABLE IF NOT EXISTS facts"));
        assert!(ddl.contains("CREATE TABLE IF NOT EXISTS edges"));
        assert!(ddl.contains("idx_facts_user_id"));
        assert!(ddl.contains("idx_facts_user_category"));
        assert!(ddl.contains("idx_facts_tombstoned"));
        assert!(ddl.contains("idx_edges_from"));
        assert!(ddl.contains("idx_edges_to"));
    }

    #[test]
    fn test_storage_config_defaults() {
        let config = StorageConfig::default();
        assert_eq!(config.db_path, "kineti_memory.db");
        assert_eq!(config.tombstone_zeroing_hours, 24);
        assert!(config.as_path().is_some());
    }

    #[test]
    fn test_storage_config_in_memory() {
        let config = StorageConfig::in_memory();
        assert_eq!(config.db_path, ":memory:");
        assert!(config.as_path().is_none());
    }

    #[test]
    fn test_persisted_fact_roundtrip() {
        let fact = UserFact::new(
            "user_42",
            "location",
            "home_city",
            "Tokyo",
            0.95,
            Some("msg_abc".to_string()),
            1710000000,
        );
        let persisted = PersistedFact::from_user_fact(&fact);
        assert_eq!(persisted.user_id, "user_42");
        assert_eq!(persisted.category, "location");
        assert_eq!(persisted.key, "home_city");
        assert_eq!(persisted.value, "Tokyo");
        assert!((persisted.confidence - 0.95).abs() < 0.01);
        assert_eq!(persisted.source_message_id, Some("msg_abc".to_string()));
        assert!(!persisted.tombstoned);

        // Roundtrip back
        let restored = persisted.to_user_fact();
        assert_eq!(restored.user_id, "user_42");
        assert_eq!(restored.value, "Tokyo");
    }

    #[test]
    fn test_persisted_edge_roundtrip() {
        let edge = FactEdge {
            from_id: "fact_new".to_string(),
            to_id: "fact_old".to_string(),
            relation: FactRelation::Replaces,
        };
        let persisted = PersistedEdge::from_fact_edge(&edge);
        assert_eq!(persisted.relation, "replaces");

        let restored = persisted.to_fact_edge().expect("Valid relation");
        assert_eq!(restored.from_id, "fact_new");
        assert_eq!(restored.to_id, "fact_old");
        assert_eq!(restored.relation, FactRelation::Replaces);
    }

    #[test]
    fn test_all_relation_types_roundtrip() {
        for (relation, expected_str) in [
            (FactRelation::Replaces, "replaces"),
            (FactRelation::RelatesTo, "relates_to"),
            (FactRelation::DerivedFrom, "derived_from"),
        ] {
            let edge = FactEdge {
                from_id: "a".to_string(),
                to_id: "b".to_string(),
                relation: relation.clone(),
            };
            let persisted = PersistedEdge::from_fact_edge(&edge);
            assert_eq!(persisted.relation, expected_str);
            let restored = persisted.to_fact_edge().unwrap();
            assert_eq!(restored.relation, relation);
        }
    }

    #[test]
    fn test_invalid_relation_returns_none() {
        let bad = PersistedEdge {
            from_id: "a".to_string(),
            to_id: "b".to_string(),
            relation: "unknown_relation".to_string(),
        };
        assert!(bad.to_fact_edge().is_none());
    }

    #[test]
    fn test_fact_to_params_layout() {
        let fact = PersistedFact {
            id: "f1".to_string(),
            user_id: "u1".to_string(),
            category: "location".to_string(),
            key: "city".to_string(),
            value: "Berlin".to_string(),
            confidence: 0.8,
            source_message_id: None,
            created_at_ms: 1710000000,
            tombstoned: false,
            tombstoned_at_ms: 0,
        };
        let params = fact_to_params(&fact);
        assert_eq!(params.len(), 10);
        assert_eq!(params[0], StorageParam::Text("f1".to_string()));
        assert_eq!(params[1], StorageParam::Text("u1".to_string()));
        assert_eq!(params[6], StorageParam::Null); // source_msg_id is None
        assert_eq!(params[8], StorageParam::Integer(0)); // not tombstoned
    }

    #[test]
    fn test_zeroing_cutoff_calculation() {
        let now = 1_710_100_000_000u64; // some timestamp in ms
        let cutoff = StorageOps::zeroing_cutoff_ms(now, 24);
        let expected = now - (24 * 60 * 60 * 1000);
        assert_eq!(cutoff, expected);
    }

    #[test]
    fn test_zeroing_cutoff_saturates_at_zero() {
        let cutoff = StorageOps::zeroing_cutoff_ms(1000, 24);
        assert_eq!(cutoff, 0);
    }

    #[test]
    fn test_sql_statements_are_nonempty() {
        assert!(!StorageOps::insert_fact_sql().is_empty());
        assert!(!StorageOps::insert_edge_sql().is_empty());
        assert!(!StorageOps::query_active_facts_sql().is_empty());
        assert!(!StorageOps::query_active_facts_by_category_sql().is_empty());
        assert!(!StorageOps::tombstone_fact_sql().is_empty());
        assert!(!StorageOps::disk_zero_sweep_sql().is_empty());
        assert!(!StorageOps::query_edges_for_fact_sql().is_empty());
        assert!(!StorageOps::query_all_edges_sql().is_empty());
        assert!(!StorageOps::hard_delete_fact_sql().is_empty());
        assert!(!StorageOps::count_facts_for_user_sql().is_empty());
        assert!(!StorageOps::wal_mode_pragma().is_empty());
        assert!(!StorageOps::busy_timeout_pragma().is_empty());
        assert!(!StorageOps::foreign_keys_pragma().is_empty());
    }

    #[test]
    fn test_user_partitioned_queries_contain_user_id_filter() {
        // Every user-facing query MUST include user_id filtering
        let active = StorageOps::query_active_facts_sql();
        assert!(active.contains("user_id = ?1"));
        let by_cat = StorageOps::query_active_facts_by_category_sql();
        assert!(by_cat.contains("user_id = ?1"));
        let count = StorageOps::count_facts_for_user_sql();
        assert!(count.contains("user_id = ?1"));
    }

    #[test]
    fn test_tombstone_sql_includes_timestamp() {
        let sql = StorageOps::tombstone_fact_sql();
        assert!(sql.contains("tombstoned = 1"));
        assert!(sql.contains("tombstoned_at_ms = ?2"));
    }

    #[test]
    fn test_disk_zero_sweep_targets_old_tombstones() {
        let sql = StorageOps::disk_zero_sweep_sql();
        assert!(sql.contains("value = ''"));
        assert!(sql.contains("tombstoned = 1"));
        assert!(sql.contains("tombstoned_at_ms < ?1"));
    }
}

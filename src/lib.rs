//! Kineti — agent harness library.
//! Context integrity, mechanical governance, chained memory (ETHOS.md).
//! The binary in main.rs is a thin CLI over this crate; tests/guarantees.rs
//! pins every standing guarantee as an executable invariant.

pub mod agent_loop;
pub mod anchor;
pub mod auth;
pub mod config;
pub mod daemon;
pub mod enforce;
pub mod ffi;
pub mod integrity;
pub mod ipc;
pub mod light;
pub mod memory;
pub mod gateway;
pub mod provider;
pub mod plan;
pub mod quarantine;
pub mod receipt;
pub mod stages;
pub mod swarm;
pub mod tools;
pub mod worktree;

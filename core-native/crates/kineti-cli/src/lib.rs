//! Native safety and reported-spending gate. Local signatures are not test trust.
pub mod compat;
pub mod gate;
pub mod init;
mod journal;
pub mod mcp;
pub mod receipt;
pub mod self_test;
pub mod spend;
pub mod store;
pub mod undo;

//! Native safety and reported-spending gate. Local signatures are not test trust.
pub mod compat;
pub mod gate;
pub mod host_control;
pub mod init;
mod journal;
pub mod mcp;
pub mod receipt;
pub mod self_test;
pub mod spend;
pub mod store;
pub mod undo;

pub mod task;
pub mod task_grant;
pub mod task_snapshot;

pub mod vm_control;

pub mod task_pipe;

pub mod task_checkpoint;
pub mod task_guest;

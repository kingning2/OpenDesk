//! 检查点适配（调用 ports::CheckpointStore）。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

mod checkpoint;
mod memory;

pub use checkpoint::{
    context_from_record, definition_from_record, now_ms, now_rfc3339, CheckpointGateway,
};
pub use memory::InMemoryCheckpointStore;

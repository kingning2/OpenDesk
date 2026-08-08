//! Workflow EventBus。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

mod bus;
mod kinds;

pub use bus::{InMemoryEventBus, WorkflowEventBus};
pub use kinds::WorkflowEvent;

//! NodeExecutor trait、Registry 与内置执行器。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

mod builtin;
mod registry;
mod traits;

pub use builtin::register_builtin_executors;
pub use registry::ExecutorRegistry;
pub use traits::{ExecuteInput, ExecuteOutput, NodeExecutor};

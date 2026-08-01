//! 内置与最小业务适配执行器。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

mod ai;
mod delay;
mod end;
mod http;
mod if_branch;
mod start;
mod switch;

use crate::error::WorkflowError;
use crate::executor::registry::ExecutorRegistry;
use std::sync::Arc;

/// 注册全部内置 + 最小业务适配 Executor。
///
/// @author coisini
/// @created 2026-07-23
///
/// @param registry - 目标注册表
/// @returns 成功或重复注册错误
pub fn register_builtin_executors(registry: &mut ExecutorRegistry) -> Result<(), WorkflowError> {
    registry.register(Arc::new(start::StartExecutor))?;
    registry.register(Arc::new(end::EndExecutor))?;
    registry.register(Arc::new(delay::DelayExecutor))?;
    registry.register(Arc::new(if_branch::IfExecutor))?;
    registry.register(Arc::new(switch::SwitchExecutor))?;
    registry.register(Arc::new(http::HttpExecutor))?;
    registry.register(Arc::new(ai::AiExecutor))?;
    // 采集节点由 app 注入真实 Executor（含 CrawlerService / LLM 依赖）
    Ok(())
}

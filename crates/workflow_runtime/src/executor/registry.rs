//! Executor Registry 与 Builder。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use super::traits::NodeExecutor;
use crate::definition::NodeType;
use crate::error::WorkflowError;
use std::collections::HashMap;
use std::sync::Arc;

/// 节点类型 → Executor。
///
/// @author Xiaoman
/// @created 2026-07-23
#[derive(Clone, Default)]
pub struct ExecutorRegistry {
    executors: HashMap<NodeType, Arc<dyn NodeExecutor>>,
}

impl ExecutorRegistry {
    /// 空注册表。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @returns 空 Registry
    pub fn new() -> Self {
        Self {
            executors: HashMap::new(),
        }
    }

    /// 注册执行器；同类型重复则报错。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @param executor - 执行器
    /// @returns 成功或 AlreadyRegistered
    pub fn register(&mut self, executor: Arc<dyn NodeExecutor>) -> Result<(), WorkflowError> {
        let node_type = executor.node_type();
        match self.executors.contains_key(&node_type) {
            true => Err(WorkflowError::ExecutorAlreadyRegistered {
                node_type: node_type.to_string(),
            }),
            false => {
                self.executors.insert(node_type, executor);
                Ok(())
            }
        }
    }

    /// 获取执行器。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @param node_type - 类型
    /// @returns Arc 或 NotRegistered
    pub fn get(&self, node_type: NodeType) -> Result<Arc<dyn NodeExecutor>, WorkflowError> {
        match self.executors.get(&node_type) {
            Some(exec) => Ok(Arc::clone(exec)),
            None => Err(WorkflowError::ExecutorNotRegistered {
                node_type: node_type.to_string(),
            }),
        }
    }
}

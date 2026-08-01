//! Workflow Runtime 唯一错误类型。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use thiserror::Error;

/// Runtime 统一错误；公共 API 一律 `Result<T, WorkflowError>`。
///
/// @author Xiaoman
/// @created 2026-07-23
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WorkflowError {
    /// 图定义不合法。
    #[error("invalid graph: {message}")]
    InvalidGraph { message: String },

    /// 状态迁移非法。
    #[error("invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    /// 节点类型未注册 Executor。
    #[error("executor not registered for node type: {node_type}")]
    ExecutorNotRegistered { node_type: String },

    /// Executor 已为该类型注册过。
    #[error("executor already registered for node type: {node_type}")]
    ExecutorAlreadyRegistered { node_type: String },

    /// 实例不存在。
    #[error("workflow instance not found: {instance_id}")]
    InstanceNotFound { instance_id: String },

    /// 节点不存在。
    #[error("node not found: {node_id}")]
    NodeNotFound { node_id: String },

    /// Context 路径无效或类型不匹配。
    #[error("context error: {message}")]
    Context { message: String },

    /// 持久化失败。
    #[error("persistence error: {message}")]
    Persistence { message: String },

    /// 节点执行失败（由 Scheduler 决定是否 Retry）。
    #[error("node execution failed: {message}")]
    NodeExecution { message: String },

    /// 操作在当前状态下不允许。
    #[error("operation not allowed: {message}")]
    NotAllowed { message: String },

    /// 内部不变量破坏。
    #[error("internal error: {message}")]
    Internal { message: String },
}

impl WorkflowError {
    /// 构造非法图错误。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @param message - 说明
    /// @returns 错误
    pub fn invalid_graph(message: impl Into<String>) -> Self {
        Self::InvalidGraph {
            message: message.into(),
        }
    }

    /// 构造持久化错误。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @param message - 说明
    /// @returns 错误
    pub fn persistence(message: impl Into<String>) -> Self {
        Self::Persistence {
            message: message.into(),
        }
    }

    /// 构造节点执行错误。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @param message - 说明
    /// @returns 错误
    pub fn node_execution(message: impl Into<String>) -> Self {
        Self::NodeExecution {
            message: message.into(),
        }
    }
}

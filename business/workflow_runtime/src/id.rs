//! 强类型 ID，禁止用裸字符串驱动业务分支。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

use serde::{Deserialize, Serialize};
use std::fmt;

/// 工作流定义 ID。
///
/// @author coisini
/// @created 2026-07-23
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowId(String);

impl WorkflowId {
    /// 从字符串构造。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @param value - 原始 id
    /// @returns 包装后的 id
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// 借用内部字符串。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @returns id 切片
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WorkflowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 节点 ID（图内唯一）。
///
/// @author coisini
/// @created 2026-07-23
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct NodeId(String);

impl NodeId {
    /// 从字符串构造。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @param value - 原始 id
    /// @returns 包装后的 id
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// 借用内部字符串。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @returns id 切片
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 运行实例 ID。
///
/// @author coisini
/// @created 2026-07-23
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstanceId(String);

impl InstanceId {
    /// 生成新的 UUID 实例 id。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @returns 新实例 id
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// 从已有字符串构造。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @param value - 原始 id
    /// @returns 包装后的 id
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// 借用内部字符串。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @returns id 切片
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

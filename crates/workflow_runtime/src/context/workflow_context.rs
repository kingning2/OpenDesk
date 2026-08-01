//! WorkflowContext 与补丁。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use crate::error::WorkflowError;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Context 补丁：按路径写入。
///
/// @author Xiaoman
/// @created 2026-07-23
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextPatch {
    /// 点分路径，如 `customer.email`。
    pub path: String,
    /// 新值。
    pub value: Value,
}

/// 全图共享 JSON 上下文。
///
/// @author Xiaoman
/// @created 2026-07-23
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowContext {
    root: Value,
    version: u64,
}

impl Default for WorkflowContext {
    fn default() -> Self {
        Self {
            root: Value::Object(Map::new()),
            version: 0,
        }
    }
}

impl WorkflowContext {
    /// 空 Context。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @returns 新 Context
    pub fn new() -> Self {
        Self::default()
    }

    /// 从 JSON 值构造。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @param root - JSON 根（应为 object）
    /// @param version - 版本
    /// @returns Context
    pub fn from_value(root: Value, version: u64) -> Self {
        let root = match root {
            Value::Object(_) => root,
            other => Value::Object(Map::from_iter([("value".to_string(), other)])),
        };
        Self { root, version }
    }

    /// 当前版本。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @returns 版本号
    pub fn version(&self) -> u64 {
        self.version
    }

    /// 序列化为 JSON 值。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @returns 根 Value
    pub fn to_value(&self) -> Value {
        self.root.clone()
    }

    /// 按路径读取。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @param path - 点分路径
    /// @returns 值或错误
    pub fn get_path(&self, path: &str) -> Result<&Value, WorkflowError> {
        let mut current = &self.root;
        for segment in path.split('.').filter(|s| !s.is_empty()) {
            match current {
                Value::Object(map) => match map.get(segment) {
                    Some(next) => current = next,
                    None => {
                        return Err(WorkflowError::Context {
                            message: format!("path not found: {path}"),
                        });
                    }
                },
                _ => {
                    return Err(WorkflowError::Context {
                        message: format!("path not an object at segment before '{segment}'"),
                    });
                }
            }
        }
        Ok(current)
    }

    /// 应用补丁并递增版本。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @param patch - 补丁
    /// @returns 成功或错误
    pub fn apply_patch(&mut self, patch: &ContextPatch) -> Result<(), WorkflowError> {
        self.set_path(&patch.path, patch.value.clone())?;
        self.version = self.version.saturating_add(1);
        Ok(())
    }

    /// 批量应用补丁（每补丁 +1 版本）。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @param patches - 补丁列表
    /// @returns 成功或错误
    pub fn apply_patches(&mut self, patches: &[ContextPatch]) -> Result<(), WorkflowError> {
        for patch in patches {
            self.apply_patch(patch)?;
        }
        Ok(())
    }

    fn set_path(&mut self, path: &str, value: Value) -> Result<(), WorkflowError> {
        let segments: Vec<&str> = path.split('.').filter(|s| !s.is_empty()).collect();
        match segments.is_empty() {
            true => {
                return Err(WorkflowError::Context {
                    message: "empty context path".to_string(),
                });
            }
            false => {}
        }

        if !self.root.is_object() {
            self.root = Value::Object(Map::new());
        }

        let mut current = &mut self.root;
        let last_index = segments.len() - 1;
        for (index, segment) in segments.iter().enumerate() {
            match index == last_index {
                true => {
                    let obj = match current.as_object_mut() {
                        Some(map) => map,
                        None => {
                            return Err(WorkflowError::Context {
                                message: format!("cannot set path {path}: not an object"),
                            });
                        }
                    };
                    obj.insert((*segment).to_string(), value);
                    return Ok(());
                }
                false => {
                    let obj = match current.as_object_mut() {
                        Some(map) => map,
                        None => {
                            return Err(WorkflowError::Context {
                                message: format!("cannot traverse path {path}"),
                            });
                        }
                    };
                    if !obj.contains_key(*segment) {
                        obj.insert((*segment).to_string(), Value::Object(Map::new()));
                    }
                    current = match obj.get_mut(*segment) {
                        Some(next) => next,
                        None => {
                            return Err(WorkflowError::Internal {
                                message: "context map missing key after insert".to_string(),
                            });
                        }
                    };
                    if !current.is_object() {
                        *current = Value::Object(Map::new());
                    }
                }
            }
        }
        Ok(())
    }
}

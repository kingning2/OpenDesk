//! EventBus trait 与内存实现。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

use super::kinds::WorkflowEvent;
use crate::error::WorkflowError;
use std::sync::{Arc, Mutex};

/// 事件总线。
///
/// @author coisini
/// @created 2026-07-23
pub trait WorkflowEventBus: Send + Sync {
    /// 发布事件。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @param event - 事件
    /// @returns 成功或错误
    fn publish(&self, event: WorkflowEvent) -> Result<(), WorkflowError>;
}

type Listener = Arc<dyn Fn(&WorkflowEvent) + Send + Sync>;

/// 内存 EventBus（测试 / 进程内适配前缓冲）。
///
/// @author coisini
/// @created 2026-07-23
#[derive(Clone, Default)]
pub struct InMemoryEventBus {
    listeners: Arc<Mutex<Vec<Listener>>>,
    history: Arc<Mutex<Vec<WorkflowEvent>>>,
}

impl InMemoryEventBus {
    /// 新建总线。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @returns 总线
    pub fn new() -> Self {
        Self::default()
    }

    /// 订阅。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @param listener - 回调
    pub fn subscribe(&self, listener: Listener) {
        match self.listeners.lock() {
            Ok(mut list) => list.push(listener),
            Err(_) => {}
        }
    }

    /// 历史事件快照（测试用）。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @returns 事件列表
    pub fn history(&self) -> Vec<WorkflowEvent> {
        match self.history.lock() {
            Ok(list) => list.clone(),
            Err(_) => Vec::new(),
        }
    }
}

impl WorkflowEventBus for InMemoryEventBus {
    fn publish(&self, event: WorkflowEvent) -> Result<(), WorkflowError> {
        match self.history.lock() {
            Ok(mut list) => list.push(event.clone()),
            Err(error) => {
                return Err(WorkflowError::Internal {
                    message: format!("event history lock poisoned: {error}"),
                });
            }
        }

        let listeners = match self.listeners.lock() {
            Ok(list) => list.clone(),
            Err(error) => {
                return Err(WorkflowError::Internal {
                    message: format!("event listeners lock poisoned: {error}"),
                });
            }
        };

        for listener in listeners {
            listener(&event);
        }
        Ok(())
    }
}

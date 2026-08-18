//! 消息去重 — 同一会话的同一内容在等待时间内不重复回复。
//!
//! 对齐 Python 版：去重键 = chat_id + send_message；等待时间默认 3600s，
//! 可由账号配置覆盖（>=60s 生效）。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 去重键。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DedupKey {
    pub chat_id: String,
    pub content: String,
}

impl DedupKey {
    pub fn new(chat_id: &str, content: &str) -> Self {
        Self {
            chat_id: chat_id.to_string(),
            content: content.to_string(),
        }
    }
}

/// 去重存储 trait — 便于未来换 Redis / DB 实现。
pub trait DedupStore: Send + Sync {
    /// 检查是否在等待时间内已处理（返回 true 表示重复）。
    fn is_processed(&self, key: &DedupKey) -> bool;
    /// 标记已处理（重置等待时间）。
    fn mark_processed(&self, key: &DedupKey);
}

/// 内存去重实现（单进程可用；多进程未来换共享存储）。
///
/// 使用 `std::sync::Mutex`：临界区仅 HashMap 读写，纳秒级，不跨 await 持有。
#[derive(Clone)]
pub struct InMemoryDedup {
    inner: Arc<Mutex<Inner>>,
    expire: Duration,
}

struct Inner {
    map: HashMap<DedupKey, Instant>,
}

impl InMemoryDedup {
    /// 指定等待窗口（秒）。业务约束（<60 用默认）由调用方处理。
    pub fn new(expire_seconds: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                map: HashMap::new(),
            })),
            expire: Duration::from_secs(expire_seconds),
        }
    }

    /// 清理过期条目（防止内存膨胀）。
    fn cleanup(&self) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        inner.map.retain(|_, last| last.elapsed() < self.expire);
    }
}

impl DedupStore for InMemoryDedup {
    fn is_processed(&self, key: &DedupKey) -> bool {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        inner
            .map
            .get(key)
            .map(|last| last.elapsed() < self.expire)
            .unwrap_or(false)
    }

    fn mark_processed(&self, key: &DedupKey) {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        inner.map.insert(key.clone(), Instant::now());
        // 容量保护：超 10k 时触发清理。
        if inner.map.len() > 10_000 {
            drop(inner);
            self.cleanup();
        }
    }
}

impl Default for InMemoryDedup {
    fn default() -> Self {
        Self::new(3600)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn duplicate_within_window() {
        let dedup = InMemoryDedup::new(3600);
        let key = DedupKey::new("chat-1", "你好");
        assert!(!dedup.is_processed(&key));
        dedup.mark_processed(&key);
        assert!(dedup.is_processed(&key));
    }

    #[test]
    fn different_content_not_duplicate() {
        let dedup = InMemoryDedup::new(3600);
        let key_a = DedupKey::new("chat-1", "你好");
        let key_b = DedupKey::new("chat-1", "多少钱");
        dedup.mark_processed(&key_a);
        assert!(!dedup.is_processed(&key_b));
    }

    #[test]
    fn expires_after_window() {
        let dedup = InMemoryDedup::new(1);
        let key = DedupKey::new("chat-1", "你好");
        dedup.mark_processed(&key);
        thread::sleep(Duration::from_millis(1500));
        assert!(!dedup.is_processed(&key));
    }

    #[test]
    fn different_chat_not_duplicate() {
        let dedup = InMemoryDedup::new(3600);
        let key_a = DedupKey::new("chat-1", "你好");
        let key_b = DedupKey::new("chat-2", "你好");
        dedup.mark_processed(&key_a);
        assert!(!dedup.is_processed(&key_b));
    }
}

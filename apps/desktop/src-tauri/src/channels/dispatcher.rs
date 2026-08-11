//! 渠道调度器 — 协议注册表 + 多账号生命周期 + 入站管线。
//!
//! 业务层通过 dispatcher 操作各平台协议，不直接依赖具体实现。

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::protocol::{ChannelAccount, ChannelKind, ChannelProtocol, ConnectionState};

/// 调度器错误。
#[derive(Debug, thiserror::Error)]
pub enum DispatcherError {
    #[error("unsupported channel kind: {0}")]
    UnsupportedKind(String),
    #[error("channel not registered: {0}")]
    NotRegistered(String),
    #[error("channel error: {0}")]
    Channel(String),
}

/// 多渠道调度器。
#[derive(Clone)]
pub struct ChannelDispatcher {
    /// kind → 协议实现。
    protocols: Arc<RwLock<HashMap<ChannelKind, Arc<dyn ChannelProtocol>>>>,
    /// account_id → 协议。
    active: Arc<RwLock<HashMap<String, Arc<dyn ChannelProtocol>>>>,
}

impl Default for ChannelDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelDispatcher {
    pub fn new() -> Self {
        Self {
            protocols: Arc::new(RwLock::new(HashMap::new())),
            active: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册平台协议。新平台接入点：实现 trait 后在此注册。
    pub fn register(&self, protocol: Arc<dyn ChannelProtocol>) {
        let kind = protocol.kind();
        let mut map = self.protocols.blocking_write();
        map.insert(kind, protocol);
    }

    async fn protocol_for(
        &self,
        account: &ChannelAccount,
    ) -> Result<Arc<dyn ChannelProtocol>, DispatcherError> {
        let kind = ChannelKind::from_str(&account.kind).ok_or_else(|| {
            DispatcherError::UnsupportedKind(account.kind.clone())
        })?;
        let map = self.protocols.read().await;
        map.get(&kind)
            .cloned()
            .ok_or_else(|| DispatcherError::NotRegistered(kind.to_string()))
    }

    /// 连接账号。
    pub async fn connect(
        &self,
        account: &ChannelAccount,
    ) -> Result<(), DispatcherError> {
        let protocol = self.protocol_for(account).await?;
        protocol
            .connect(account)
            .await
            .map_err(|error| DispatcherError::Channel(error.to_string()))?;
        self.active
            .write()
            .await
            .insert(account.id.clone(), protocol);
        Ok(())
    }

    /// 断开账号。
    pub async fn disconnect(&self, account_id: &str) -> Result<(), DispatcherError> {
        if let Some(protocol) = self.active.write().await.remove(account_id) {
            protocol
                .disconnect()
                .await
                .map_err(|error| DispatcherError::Channel(error.to_string()))?;
        }
        Ok(())
    }

    /// 发送消息；`peer_id` 为平台侧会话/对方 id。
    pub async fn send(
        &self,
        account_id: &str,
        peer_id: &str,
        text: &str,
    ) -> Result<String, DispatcherError> {
        let protocol = self
            .active
            .read()
            .await
            .get(account_id)
            .cloned()
            .ok_or_else(|| DispatcherError::NotRegistered(account_id.to_string()))?;
        protocol
            .send(peer_id, text)
            .await
            .map_err(|error| DispatcherError::Channel(error.to_string()))
    }

    /// 查询连接状态。
    pub async fn connection_state(&self, account_id: &str) -> ConnectionState {
        match self.active.read().await.get(account_id) {
            Some(protocol) => protocol.connection_state(),
            None => ConnectionState::Disconnected,
        }
    }
}

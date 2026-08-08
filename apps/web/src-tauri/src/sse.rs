//! SSE 事件中枢与 emitter 桥（替代桌面的 Tauri `app.emit`）。

use chat::ChatUIEmitter;
use common::contracts::{
    ChatEventToken, ChatEventTool, CrawlerEventChannelAccepted, CrawlerEventJobCompleted,
    CrawlerEventJobFailed, CrawlerEventJobLog, CrawlerEventJobProgress, CrawlerEventJobStarted,
};
use crawler::youtube::{CrawlerUIEmitter, CrawlerUIEvent};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// SSE 广播中枢：多个订阅者频道，广播时 try_send，背压（频道满）丢弃最旧。
#[derive(Clone, Default)]
pub struct SseHub {
    subscribers: Arc<RwLock<HashMap<u64, mpsc::Sender<Arc<str>>>>>,
    next_id: Arc<AtomicU64>,
}

impl SseHub {
    /// 新建中枢。
    pub fn new() -> Self {
        Self::default()
    }

    /// 订阅：返回可异步读取的频道与订阅 id。
    pub async fn subscribe(&self) -> (mpsc::Receiver<Arc<str>>, u64) {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::channel(128);
        self.subscribers.write().await.insert(id, tx);
        (rx, id)
    }

    /// 退订。
    pub async fn unsubscribe(&self, id: u64) {
        self.subscribers.write().await.remove(&id);
    }

    /// 向所有订阅者广播一条已编码的 SSE 帧字符串；频道满则丢弃该条。
    pub async fn broadcast(&self, frame: Arc<str>) {
        let subscribers = self.subscribers.read().await.clone();
        for tx in subscribers.values() {
            let _ = tx.try_send(frame.clone());
        }
    }
}

/// 把事件编码为 SSE 帧：`event: <topic>\ndata: <json>\n\n`。
pub fn encode_frame(topic: &str, payload: &impl serde::Serialize) -> Arc<str> {
    let json = serde_json::to_string(payload).unwrap_or_else(|_| "{}".to_string());
    Arc::from(format!("event: {topic}\ndata: {json}\n\n"))
}

/// 桥接 chat 流式 token/tool 到 SSE。
#[derive(Clone)]
pub struct SseChatEmitter {
    hub: SseHub,
}

impl SseChatEmitter {
    /// 绑定中枢。
    pub fn new(hub: SseHub) -> Self {
        Self { hub }
    }

    fn emit<T: serde::Serialize>(&self, topic: &str, payload: &T) {
        let hub = self.hub.clone();
        let frame = encode_frame(topic, payload);
        tokio::spawn(async move {
            hub.broadcast(frame).await;
        });
    }
}

impl ChatUIEmitter for SseChatEmitter {
    fn emit_message_token(&self, event: &ChatEventToken) {
        self.emit("chat:message/token", event);
    }

    fn emit_message_tool(&self, event: &ChatEventTool) {
        self.emit("chat:message/tool", event);
    }
}

/// 桥接爬虫 job / channel 事件到 SSE。
#[derive(Clone)]
pub struct SseCrawlerEmitter {
    hub: SseHub,
}

impl SseCrawlerEmitter {
    /// 绑定中枢。
    pub fn new(hub: SseHub) -> Self {
        Self { hub }
    }

    fn emit<T: serde::Serialize>(&self, event: CrawlerUIEvent, payload: &T) {
        let hub = self.hub.clone();
        let frame = encode_frame(event.as_str(), payload);
        tokio::spawn(async move {
            hub.broadcast(frame).await;
        });
    }
}

impl CrawlerUIEmitter for SseCrawlerEmitter {
    fn emit_job_started(&self, event: &CrawlerEventJobStarted) {
        self.emit(CrawlerUIEvent::JobStarted, event);
    }

    fn emit_job_progress(&self, event: &CrawlerEventJobProgress) {
        self.emit(CrawlerUIEvent::JobProgress, event);
    }

    fn emit_job_log(&self, event: &CrawlerEventJobLog) {
        self.emit(CrawlerUIEvent::JobLog, event);
    }

    fn emit_job_completed(&self, event: &CrawlerEventJobCompleted) {
        self.emit(CrawlerUIEvent::JobCompleted, event);
    }

    fn emit_job_failed(&self, event: &CrawlerEventJobFailed) {
        self.emit(CrawlerUIEvent::JobFailed, event);
    }

    fn emit_channel_accepted(&self, event: &CrawlerEventChannelAccepted) {
        self.emit(CrawlerUIEvent::ChannelAccepted, event);
    }
}

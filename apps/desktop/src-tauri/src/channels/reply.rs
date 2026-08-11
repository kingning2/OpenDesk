//! 自动回复决策 — 意图路由 + 上下文组织 + LLM 调用。
//!
//! 业务调度全在 Rust：收到入站消息后，规则优先判断意图，兜底调 Python `llm/classify`；
//! 组上下文（商品信息 + 会话历史）后调 Python `llm/chat` 生成回复。

use common::contracts::{
    ChannelConversation, ChannelMessage, LlmIpcChatRequest, LlmIpcChatResponse,
    LlmIpcClassifyRequest, LlmIpcClassifyResponse, LlmMessage, LlmProvider,
};
use runtime::sidecar::client::SidecarClient;
use runtime::sidecar::routes;

/// 意图结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    /// 无需回复（问候/结束语等）。
    NoReply,
    /// 价格/议价。
    Price,
    /// 技术/商品咨询。
    Tech,
    /// 默认客服回复。
    Default,
}

impl Intent {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Intent::NoReply => "no_reply",
            Intent::Price => "price",
            Intent::Tech => "tech",
            Intent::Default => "default",
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "no_reply" => Intent::NoReply,
            "price" => Intent::Price,
            "tech" => Intent::Tech,
            _ => Intent::Default,
        }
    }
}

/// 技术意图关键词（优先判定）。
const TECH_KEYWORDS: &[&str] = &["参数", "规格", "型号", "连接", "对比"];
/// 价格意图关键词。
const PRICE_KEYWORDS: &[&str] = &["便宜", "价", "砍价", "少点"];
/// 价格意图正则。
const PRICE_PATTERNS: &[&str] = &[r"\d+元", r"能少\d+"];
/// 无需回复关键词。
const NO_REPLY_KEYWORDS: &[&str] = &["谢谢", "好的", "嗯嗯", "再见", "没了", "不需要了"];

/// 纯规则意图路由（可单测，不依赖 LLM）。
pub fn route_intent(text: &str) -> Intent {
    // 去除标点后匹配。
    let clean: String = text
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '元' || c.is_ascii_digit())
        .collect();

    if NO_REPLY_KEYWORDS.iter().any(|kw| clean.contains(kw)) {
        return Intent::NoReply;
    }
    if TECH_KEYWORDS.iter().any(|kw| clean.contains(kw)) {
        return Intent::Tech;
    }
    for pattern in PRICE_PATTERNS {
        if regex_match(pattern, &clean) {
            return Intent::Price;
        }
    }
    if PRICE_KEYWORDS.iter().any(|kw| clean.contains(kw)) {
        return Intent::Price;
    }
    Intent::Default
}

fn regex_match(pattern: &str, text: &str) -> bool {
    // 简化：仅支持字面量包含判断（`\d+元` → 含"元"且含数字；`能少\d+` → 前缀"能少"后跟数字）。
    match pattern {
        r"\d+元" => text.contains('元') && text.chars().any(|c| c.is_ascii_digit()),
        r"能少\d+" => text
            .strip_prefix("能少")
            .is_some_and(|rest| rest.chars().any(|c| c.is_ascii_digit())),
        _ => false,
    }
}

/// 回复上下文构建。
pub fn build_reply_context(
    conversation: Option<&ChannelConversation>,
    history: &[ChannelMessage],
    max_history: usize,
) -> Vec<LlmMessage> {
    let mut messages: Vec<LlmMessage> = Vec::new();

    let mut item_desc = String::new();
    if let Some(cv) = conversation {
        if let Some(title) = cv.item_title.as_deref() {
            item_desc.push_str("商品标题：");
            item_desc.push_str(title);
            item_desc.push('\n');
        }
        if let Some(price) = cv.item_price {
            item_desc.push_str("商品价格：");
            item_desc.push_str(&format!("{price} 元"));
            item_desc.push('\n');
        }
        if !item_desc.is_empty() {
            messages.push(LlmMessage {
                role: "system".to_string(),
                content: format!("你是闲鱼卖家客服。以下是当前商品信息：\n{item_desc}"),
            });
        }
    }

    // 历史截断：取最近 max_history 条。
    let start = history.len().saturating_sub(max_history);
    for msg in &history[start..] {
        let role = if msg.direction == "in" { "user" } else { "assistant" };
        messages.push(LlmMessage {
            role: role.to_string(),
            content: msg.content.clone(),
        });
    }
    messages
}

/// 回复协调器：组合规则路由 + sidecar LLM 调用。
pub struct ReplyCoordinator {
    sidecar: SidecarClient,
}

impl ReplyCoordinator {
    pub fn new(sidecar: SidecarClient) -> Self {
        Self { sidecar }
    }

    /// 生成回复。返回 `None` 表示判定无需回复。
    ///
    /// `classify` 兜底：规则未命中时调 LLM 分类；`chat` 生成实际回复。
    pub async fn generate_reply(
        &self,
        user_msg: &str,
        conversation: Option<&ChannelConversation>,
        history: &[ChannelMessage],
        provider: Option<&LlmProvider>,
        trace_id: &str,
    ) -> Result<Option<LlmIpcChatResponse>, String> {
        let intent = self.resolve_intent(user_msg, provider, trace_id).await?;
        if intent == Intent::NoReply {
            return Ok(None);
        }

        let provider = provider.cloned().unwrap_or_else(|| LlmProvider {
            base_url: String::new(),
            api_key: String::new(),
            model: String::new(),
        });
        let messages = build_reply_context(conversation, history, 20);
        let mut messages = messages;
        messages.push(LlmMessage {
            role: "user".to_string(),
            content: user_msg.to_string(),
        });

        let request = LlmIpcChatRequest {
            messages,
            provider,
            trace_id: Some(trace_id.to_string()),
        };
        let response = routes::llm_chat::call(&self.sidecar, request)
            .await
            .map_err(|error| error.to_string())?;
        Ok(Some(response))
    }

    /// 意图解析：规则优先 → LLM 兜底。
    async fn resolve_intent(
        &self,
        text: &str,
        provider: Option<&LlmProvider>,
        trace_id: &str,
    ) -> Result<Intent, String> {
        let rule_intent = route_intent(text);
        if rule_intent != Intent::Default {
            return Ok(rule_intent);
        }

        // 规则未命中 → 调 LLM 分类兜底（无 provider 则回落默认）。
        let Some(provider) = provider else {
            return Ok(Intent::Default);
        };
        let request = LlmIpcClassifyRequest {
            text: text.to_string(),
            scenario: Some("xianyu_customer_service".to_string()),
            options: vec![
                "price".to_string(),
                "tech".to_string(),
                "default".to_string(),
                "no_reply".to_string(),
            ],
            provider: Some(provider.clone()),
            trace_id: Some(trace_id.to_string()),
        };
        let response: LlmIpcClassifyResponse =
            routes::llm_classify::call(&self.sidecar, request)
                .await
                .map_err(|error| error.to_string())?;
        Ok(Intent::from_str(&response.intent))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(direction: &str, content: &str) -> ChannelMessage {
        ChannelMessage {
            id: "1".into(),
            conversation_id: "cv".into(),
            direction: direction.into(),
            sender: "s".into(),
            content: content.into(),
            created_at: "0".into(),
        }
    }

    #[test]
    fn routes_tech_intent() {
        assert_eq!(route_intent("这个型号支持什么接口"), Intent::Tech);
        assert_eq!(route_intent("参数表发一下"), Intent::Tech);
    }

    #[test]
    fn routes_price_intent() {
        assert_eq!(route_intent("能便宜点吗"), Intent::Price);
        assert_eq!(route_intent("100元可以吗"), Intent::Price);
        assert_eq!(route_intent("能少50吗"), Intent::Price);
    }

    #[test]
    fn routes_no_reply_and_default() {
        assert_eq!(route_intent("谢谢"), Intent::NoReply);
        assert_eq!(route_intent("在吗"), Intent::Default);
    }

    #[test]
    fn builds_context_with_item_and_history() {
        let cv = ChannelConversation {
            id: "cv".into(),
            account_id: "a".into(),
            peer_id: "p".into(),
            peer_name: None,
            item_id: Some("i".into()),
            item_title: Some("二手电脑".into()),
            item_price: Some(100),
            updated_at: "0".into(),
        };
        let history = vec![msg("in", "你好"), msg("out", "在的")];
        let ctx = build_reply_context(Some(&cv), &history, 20);
        assert!(ctx[0].content.contains("二手电脑"));
        assert!(ctx.iter().any(|m| m.content == "你好"));
        assert!(ctx.iter().any(|m| m.content == "在的"));
    }
}

//! 发货内容生成 — 按卡券类型生成文本内容。
//!
//! 对齐 Python 版 `build_delivery_content`：
//! - text：固定文字；
//! - data：消费一条批量数据（由数据源提供，行锁由存储实现保证）；
//! - api：调用外部接口拉取（响应字段提取 + 重试）；
//! - image：图片 URL 拼接。
//!
//! 备注（description）支持 `{DELIVERY_CONTENT}` 与订单上下文变量替换。

use super::card::Card;

/// 内容生成上下文（变量替换 / API 参数）。
#[derive(Debug, Clone, Default)]
pub struct ContentContext {
    pub order_id: String,
    pub item_id: String,
    pub buyer_id: String,
    pub spec_name: String,
    pub spec_value: String,
    pub order_amount: String,
    pub order_quantity: String,
}

impl ContentContext {
    /// 替换文本中的订单上下文变量 `{order_id}` / `{item_id}` 等。
    pub fn replace_vars(&self, text: &str) -> String {
        text.replace("{order_id}", &self.order_id)
            .replace("{item_id}", &self.item_id)
            .replace("{buyer_id}", &self.buyer_id)
            .replace("{spec_name}", &self.spec_name)
            .replace("{spec_value}", &self.spec_value)
            .replace("{order_amount}", &self.order_amount)
            .replace("{order_quantity}", &self.order_quantity)
    }
}

/// 生成的发货内容（文本 + 图片 URL 列表）。
#[derive(Debug, Clone)]
pub struct DeliveryContent {
    pub text: String,
    pub image_urls: Vec<String>,
}

impl DeliveryContent {
    /// 拼接为最终文本：文本部分 + 图片 URL 行。
    pub fn to_text(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if !self.text.trim().is_empty() {
            parts.push(self.text.trim().to_string());
        }
        if !self.image_urls.is_empty() {
            parts.push(self.image_urls.join("\n"));
        }
        parts.join("\n")
    }
}

/// 解析卡券多图片 URL（JSON 数组字符串）。
fn parse_image_urls(card: &Card) -> Vec<String> {
    if !card.image_urls.trim().is_empty() {
        if let Ok(parsed) = serde_json::from_str::<Vec<String>>(&card.image_urls) {
            return parsed.into_iter().filter(|url| !url.is_empty()).collect();
        }
    }
    if !card.image_url.trim().is_empty() {
        return vec![card.image_url.clone()];
    }
    Vec::new()
}

/// 处理备注：`{DELIVERY_CONTENT}` 替换为内容主体，其余变量替换。
fn with_description(content: &str, description: &str, context: &ContentContext) -> String {
    // 内容主体与备注各自做变量替换。
    let content = context.replace_vars(content.trim());
    let description = context.replace_vars(description.trim());
    if description.is_empty() {
        return content;
    }
    if content.is_empty() {
        return description;
    }
    description.replace("{DELIVERY_CONTENT}", &content)
}

/// 内容生成器。
pub struct ContentGenerator;

impl ContentGenerator {
    /// 生成发货内容。失败返回 `None`（data 已用完 / api 拉取失败 / 无可发内容）。
    ///
    /// `consume_data` 为 data 类型卡券的消费回调（存储实现行锁消费一条）。
    pub fn generate(
        card: &Card,
        context: &ContentContext,
        consume_data: impl FnOnce() -> Option<String>,
    ) -> Option<DeliveryContent> {
        // 1. 按类型取文本主体。
        let text = match card.card_type.as_str() {
            "text" => Some(card.text_content.clone()),
            "data" => consume_data(),
            "api" => None, // API 拉取由 gateway 层注入（见 `generate_with_api`）
            "image" => None,
            other => {
                tracing::warn!(card_id = card.id, card_type = %other, "不支持的卡券类型");
                return None;
            }
        };
        Self::assemble(card, context, text)
    }

    /// 生成发货内容（含 API 类型拉取）。
    pub async fn generate_with_api(
        card: &Card,
        context: &ContentContext,
        consume_data: impl FnOnce() -> Option<String>,
        fetch_api: impl Fn(&str) -> Option<String>,
    ) -> Option<DeliveryContent> {
        let text = match card.card_type.as_str() {
            "text" => Some(card.text_content.clone()),
            "data" => consume_data(),
            "api" => fetch_api(&card.api_config),
            "image" => None,
            other => {
                tracing::warn!(card_id = card.id, card_type = %other, "不支持的卡券类型");
                return None;
            }
        };
        Self::assemble(card, context, text)
    }

    fn assemble(
        card: &Card,
        context: &ContentContext,
        text: Option<String>,
    ) -> Option<DeliveryContent> {
        // 2. 备注与上下文处理。
        let text_part = match text {
            Some(text) => with_description(&text, &card.description, context),
            None => {
                if card.description.trim().is_empty() {
                    String::new()
                } else {
                    context.replace_vars(card.description.trim())
                }
            }
        };

        // 3. 图片 URL 收集。
        let image_urls = parse_image_urls(card);

        // 4. 无任何可发内容 → 失败。
        if text_part.trim().is_empty() && image_urls.is_empty() {
            tracing::warn!(card_id = card.id, "卡券没有可发货的内容");
            return None;
        }

        Some(DeliveryContent {
            text: text_part,
            image_urls,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(card_type: &str, content: &str) -> Card {
        Card {
            id: 1,
            owner_id: 0,
            account_id: String::new(),
            name: "卡".to_string(),
            card_type: card_type.to_string(),
            source: super::super::card::CardSource::Own,
            enabled: true,
            text_content: content.to_string(),
            data_content: "data1\ndata2".to_string(),
            image_url: String::new(),
            image_urls: String::new(),
            api_config: String::new(),
            delay_seconds: 0,
            description: String::new(),
        }
    }

    #[test]
    fn text_card_uses_fixed_content() {
        let c = card("text", "欢迎购买");
        let ctx = ContentContext::default();
        let content = ContentGenerator::generate(&c, &ctx, || None).expect("content");
        assert_eq!(content.to_text(), "欢迎购买");
    }

    #[test]
    fn data_card_consumes_one_line() {
        let c = card("data", "");
        let ctx = ContentContext::default();
        let content =
            ContentGenerator::generate(&c, &ctx, || Some("data1".to_string())).expect("content");
        assert_eq!(content.text, "data1");
    }

    #[test]
    fn data_card_none_when_exhausted() {
        let c = card("data", "");
        let ctx = ContentContext::default();
        assert!(ContentGenerator::generate(&c, &ctx, || None).is_none());
    }

    #[test]
    fn image_card_returns_url() {
        let mut c = card("image", "");
        c.image_url = "https://x/y.png".to_string();
        let ctx = ContentContext::default();
        let content = ContentGenerator::generate(&c, &ctx, || None).expect("content");
        assert_eq!(content.image_urls, vec!["https://x/y.png"]);
    }

    #[test]
    fn description_vars_replaced() {
        let mut c = card("text", "卡密{order_id}");
        c.description = "{DELIVERY_CONTENT} - 订单{order_id}".to_string();
        let ctx = ContentContext {
            order_id: "O-1".to_string(),
            ..Default::default()
        };
        let content = ContentGenerator::generate(&c, &ctx, || None).expect("content");
        assert_eq!(content.to_text(), "卡密O-1 - 订单O-1");
    }

    #[tokio::test]
    async fn api_card_uses_fetched() {
        let c = card("api", "");
        let ctx = ContentContext::default();
        let content =
            ContentGenerator::generate_with_api(&c, &ctx, || None, |_| Some("API卡密".to_string()))
                .await;
        assert_eq!(content.expect("content").text, "API卡密");
    }

    #[test]
    fn unsupported_type_returns_none() {
        let c = card("video", "");
        let ctx = ContentContext::default();
        assert!(ContentGenerator::generate(&c, &ctx, || None).is_none());
    }
}

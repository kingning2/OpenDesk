//! 知识库 — 商品信息提取与注入。
//!
//! 渠道无关：接收结构化的商品信息（标题/价格/描述/特殊说明），
//! 生成供 LLM 使用的可读商品上下文。

/// 商品知识（业务层从存储/渠道注入）。
#[derive(Debug, Clone, Default)]
pub struct ItemKnowledge {
    pub title: String,
    pub price: Option<f64>,
    /// 商品描述（可含 JSON 结构，需清洗）。
    pub desc: String,
    /// 商品特殊说明（AI 提示词）。
    pub ai_prompt: String,
}

/// 清洗商品描述：JSON 中提取可读文本；纯文本截断。
fn extract_readable_desc(desc: &str) -> String {
    let text = desc.trim();
    if text.is_empty() {
        return "暂无商品描述".to_string();
    }

    // 尝试 JSON 解析：取 description / desc / item_description / itemDesc / content。
    if text.starts_with('{') || text.starts_with('[') {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
            let readable = [
                "description",
                "desc",
                "item_description",
                "itemDesc",
                "content",
            ]
            .iter()
            .find_map(|key| value.get(*key).and_then(serde_json::Value::as_str))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && !s.starts_with('{') && !s.starts_with('['));
            if let Some(readable) = readable {
                return shorten(&readable, 800);
            }
            // detail_params 兜底。
            if let Some(detail) = value.get("detail_params").and_then(|v| v.as_object()) {
                let parts: Vec<&str> = ["title", "postInfo"]
                    .iter()
                    .filter_map(|key| detail.get(*key).and_then(serde_json::Value::as_str))
                    .collect();
                if !parts.is_empty() {
                    return shorten(&parts.join("，"), 800);
                }
            }
        }
        return "暂无商品描述".to_string();
    }

    shorten(text, 800)
}

/// 截断文本（超长加省略号）。
fn shorten(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let truncated: String = text.chars().take(limit).collect();
    format!("{truncated}...")
}

impl ItemKnowledge {
    /// 生成为 LLM 使用的商品上下文文本。
    pub fn to_context(&self) -> String {
        let mut context = String::new();
        let title = if self.title.trim().is_empty() {
            "未知"
        } else {
            self.title.trim()
        };
        context.push_str(&format!("商品标题: {}\n", shorten(title, 120)));
        let price = self
            .price
            .map(|p| format!("{p}"))
            .unwrap_or_else(|| "未知".to_string());
        context.push_str(&format!("商品价格: {price}元\n"));
        context.push_str(&format!("商品描述: {}", extract_readable_desc(&self.desc)));
        if !self.ai_prompt.trim().is_empty() {
            context.push_str(&format!(
                "\n商品特殊说明: {}",
                shorten(self.ai_prompt.trim(), 400)
            ));
        }
        context
    }
}

/// 便捷：从商品知识构建上下文（供业务层调用）。
pub fn build_item_context(item: &ItemKnowledge) -> String {
    item.to_context()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_desc_is_shortened() {
        let long = "长".repeat(1000);
        let item = ItemKnowledge {
            desc: long,
            ..Default::default()
        };
        let context = item.to_context();
        assert!(context.contains("商品描述: 长"));
        // 截断后字符数应远小于原文 1000 字符。
        assert!(context.chars().count() < 1000);
    }

    #[test]
    fn json_desc_extracts_readable() {
        let item = ItemKnowledge {
            title: "二手电脑".to_string(),
            price: Some(100.0),
            desc: r#"{"description":"九成新","title":"title"}"#.to_string(),
            ..Default::default()
        };
        let context = item.to_context();
        assert!(context.contains("九成新"));
        assert!(context.contains("商品标题: 二手电脑"));
        assert!(context.contains("商品价格: 100元"));
    }

    #[test]
    fn empty_desc_falls_back() {
        let item = ItemKnowledge::default();
        assert!(item.to_context().contains("暂无商品描述"));
    }

    #[test]
    fn includes_ai_prompt() {
        let item = ItemKnowledge {
            title: "x".to_string(),
            ai_prompt: "不议价".to_string(),
            ..Default::default()
        };
        assert!(item.to_context().contains("商品特殊说明: 不议价"));
    }
}

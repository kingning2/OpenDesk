//! prompt crate — 提示词模板（主流风格：模板独立文件 + 按意图字符串索引 + 变量插值）。
//!
//! 模板在 `templates/`：`system_{bargain,tech,default}.txt`（系统提示词）+ `user.txt`（用户消息）。
//! 按意图字符串索引（`"price"` / `"tech"` / 其余走 default），不依赖 agent 的 `Intent` 枚举。

use serde_json::Value;

/// 输出约束：只输出最终回复文本（所有默认提示词统一追加）。
pub const DIRECT_RULE: &str =
    "重要：只输出给买家的最终回复文本，不要输出思考过程、分析过程或解释，回复控制在40字以内。";

/// 默认系统提示词模板（按意图字符串索引；未知意图走 default）。
pub fn default_prompt(intent: &str) -> &'static str {
    match intent {
        "price" | "bargain" => include_str!("templates/system_bargain.txt"),
        "tech" => include_str!("templates/system_tech.txt"),
        _ => include_str!("templates/system_default.txt"),
    }
}

/// 提示词构建器 — 组装 system + user 消息。
pub struct PromptBuilder;

impl PromptBuilder {
    /// 生成系统提示词：自定义优先，否则默认模板 + 输出约束。
    pub fn system_prompt(intent: &str, custom_prompts: &Value) -> String {
        let custom = custom_prompts
            .get(intent)
            .and_then(Value::as_str)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let base = custom.unwrap_or_else(|| default_prompt(intent).to_string());
        format!("{base}\n{DIRECT_RULE}")
    }

    /// 组装用户消息：商品信息 + 对话历史 + 议价设置 + 当前消息。
    pub fn user_prompt(
        item_context: &str,
        history: &str,
        bargain_count: u32,
        max_bargain_rounds: u32,
        max_discount_percent: u32,
        max_discount_amount: u32,
        user_message: &str,
    ) -> String {
        include_str!("templates/user.txt")
            .replace("{item_context}", item_context)
            .replace("{history}", history)
            .replace("{bargain_count}", &bargain_count.to_string())
            .replace("{max_bargain_rounds}", &max_bargain_rounds.to_string())
            .replace("{max_discount_percent}", &max_discount_percent.to_string())
            .replace("{max_discount_amount}", &max_discount_amount.to_string())
            .replace("{user_message}", user_message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_prompt_exists_for_known_intents() {
        for intent in ["price", "tech", "default", "no_reply"] {
            assert!(!default_prompt(intent).is_empty());
        }
    }

    #[test]
    fn custom_prompt_overrides_default() {
        let custom = serde_json::json!({ "price": "你是砍价高手" });
        let system = PromptBuilder::system_prompt("price", &custom);
        assert!(system.starts_with("你是砍价高手"));
        assert!(system.contains(DIRECT_RULE));
    }

    #[test]
    fn fallback_to_default_when_no_custom() {
        let custom = serde_json::json!({});
        let system = PromptBuilder::system_prompt("price", &custom);
        assert!(system.contains("销售专家"));
    }

    #[test]
    fn user_prompt_contains_all_sections() {
        let prompt =
            PromptBuilder::user_prompt("商品标题: 测试", "user: 在吗", 1, 3, 10, 100, "便宜点");
        assert!(prompt.contains("商品信息"));
        assert!(prompt.contains("对话历史"));
        assert!(prompt.contains("议价设置"));
        assert!(prompt.contains("便宜点"));
    }
}

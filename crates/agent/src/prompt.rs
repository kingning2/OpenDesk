//! 提示词模板与组装。
//!
//! 默认按意图提供议价 / 技术 / 客服三类系统提示词；
//! 支持业务侧自定义提示词（按意图覆盖）。

use serde_json::Value;

use crate::intent::Intent;

/// 输出约束：只输出最终回复文本（所有默认提示词统一追加）。
const DIRECT_RULE: &str =
    "重要：只输出给买家的最终回复文本，不要输出思考过程、分析过程或解释，回复控制在40字以内。";

/// 默认提示词模板（对齐 Python 版 ai_reply_engine 默认提示词）。
pub fn default_prompt(intent: Intent) -> &'static str {
    match intent {
        Intent::Price => {
            "你是一位经验丰富的销售专家，擅长议价。\n语言要求：简短直接，每句≤10字，总字数≤40字。\n议价策略：\n1. 根据议价次数递减优惠：第1次小幅优惠，第2次中等优惠，第3次最大优惠\n2. 接近最大议价轮数时要坚持底线，强调商品价值\n3. 优惠不能超过设定的最大百分比和金额\n4. 语气要友好但坚定，突出商品优势\n注意：结合商品信息、对话历史和议价设置，给出合适的回复。"
        }
        Intent::Tech => {
            "你是一位技术专家，专业解答产品相关问题。\n语言要求：简短专业，每句≤10字，总字数≤40字。\n回答重点：产品功能、使用方法、注意事项。\n注意：基于商品信息回答，避免过度承诺。"
        }
        _ => {
            "你是一位资深电商卖家，提供优质客服。\n语言要求：简短友好，每句≤10字，总字数≤40字。\n回答重点：商品介绍、物流、售后等常见问题。\n注意：结合商品信息，给出实用建议。"
        }
    }
}

/// 提示词构建器 — 组装 system + user 消息。
pub struct PromptBuilder;

impl PromptBuilder {
    /// 生成系统提示词：自定义优先，否则默认 + 输出约束。
    pub fn system_prompt(intent: Intent, custom_prompts: &Value) -> String {
        let custom = custom_prompts
            .get(intent.as_str())
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
        format!(
            "商品信息：\n{item_context}\n\n对话历史：\n{history}\n\n议价设置：\n- 当前议价次数：{bargain_count}\n- 最大议价轮数：{max_bargain_rounds}\n- 最大优惠百分比：{max_discount_percent}%\n- 最大优惠金额：{max_discount_amount}元\n\n用户消息：{user_message}\n\n请根据以上信息生成回复："
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_prompt_exists_for_all_intents() {
        for intent in [
            Intent::Price,
            Intent::Tech,
            Intent::Default,
            Intent::NoReply,
        ] {
            assert!(!default_prompt(intent).is_empty());
        }
    }

    #[test]
    fn custom_prompt_overrides_default() {
        let custom = serde_json::json!({ "price": "你是砍价高手" });
        let system = PromptBuilder::system_prompt(Intent::Price, &custom);
        assert!(system.starts_with("你是砍价高手"));
        assert!(system.contains(DIRECT_RULE));
    }

    #[test]
    fn fallback_to_default_when_no_custom() {
        let custom = serde_json::json!({});
        let system = PromptBuilder::system_prompt(Intent::Price, &custom);
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

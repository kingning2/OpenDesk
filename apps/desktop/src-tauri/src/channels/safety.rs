//! 出站安全过滤 — 屏蔽诱导站外交易等敏感词。

/// 命中安全词时返回的统一提示。
pub const SAFETY_FILTERED_REPLY: &str = "[安全提醒]请通过平台沟通";

/// 敏感词表（参考 XianyuAutoAgent 的 `_safe_filter`）。
const BLOCKED_PHRASES: &[&str] = &[
    "微信",
    "QQ",
    "支付宝",
    "银行卡",
    "线下",
    "vx",
    "wx",
    "qq",
    "zfb",
];

/// 安全过滤：文本命中敏感词时替换为安全提示；否则原样返回。
pub fn filter_reply(text: &str) -> String {
    let lower = text.to_lowercase();
    if BLOCKED_PHRASES.iter().any(|phrase| lower.contains(phrase)) {
        SAFETY_FILTERED_REPLY.to_string()
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_wechat_and_money_channel_words() {
        assert_eq!(
            filter_reply("加我微信详聊"),
            SAFETY_FILTERED_REPLY.to_string()
        );
        assert_eq!(filter_reply("QQ联系"), SAFETY_FILTERED_REPLY.to_string());
        assert_eq!(filter_reply("支付宝付款"), SAFETY_FILTERED_REPLY.to_string());
    }

    #[test]
    fn passes_clean_replies() {
        assert_eq!(filter_reply("这个可以便宜点吗"), "这个可以便宜点吗");
        assert_eq!(filter_reply("包邮吗"), "包邮吗");
    }

    #[test]
    fn case_insensitive_latin() {
        assert_eq!(filter_reply("加VX"), SAFETY_FILTERED_REPLY.to_string());
    }
}

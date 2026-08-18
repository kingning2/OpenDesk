//! 本地意图检测 — 关键词规则优先，LLM 兜底由上层决定。
//!
//! 意图：price（议价）/ tech（技术咨询）/ default（默认客服）/ no_reply（无需回复）。

/// 意图类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    NoReply,
    Price,
    Tech,
    Default,
}

impl Intent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Intent::NoReply => "no_reply",
            Intent::Price => "price",
            Intent::Tech => "tech",
            Intent::Default => "default",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        match value {
            "no_reply" => Intent::NoReply,
            "price" => Intent::Price,
            "tech" => Intent::Tech,
            _ => Intent::Default,
        }
    }
}

/// 无需回复关键词（先判定）。
const NO_REPLY_KEYWORDS: &[&str] = &[
    "谢谢",
    "好的",
    "嗯嗯",
    "再见",
    "没了",
    "不需要了",
    "收到",
    "好的谢谢",
    "ok",
];
/// 技术意图关键词。
const TECH_KEYWORDS: &[&str] = &[
    "怎么用",
    "参数",
    "坏了",
    "故障",
    "设置",
    "说明书",
    "功能",
    "用法",
    "教程",
    "驱动",
    "规格",
    "型号",
    "接口",
];
/// 价格意图关键词。
const PRICE_KEYWORDS: &[&str] = &[
    "便宜",
    "优惠",
    "刀",
    "降价",
    "价格",
    "多少钱",
    "能少",
    "还能",
    "最低",
    "底价",
    "实诚价",
    "包个邮",
    "砍价",
    "少点",
];

/// 纯规则意图路由（可单测，不依赖 LLM）。
pub fn route_intent(text: &str) -> Intent {
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
    // 价格正则：`数字+元` 或 `能少+数字`。
    let has_price_pattern = (clean.contains('元') && clean.chars().any(|c| c.is_ascii_digit()))
        || (clean
            .strip_prefix("能少")
            .is_some_and(|rest| rest.chars().any(|c| c.is_ascii_digit())));
    if has_price_pattern {
        return Intent::Price;
    }
    if PRICE_KEYWORDS.iter().any(|kw| clean.contains(kw)) {
        return Intent::Price;
    }
    Intent::Default
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_tech_intent() {
        assert_eq!(route_intent("这个型号支持什么接口"), Intent::Tech);
        assert_eq!(route_intent("参数表发一下"), Intent::Tech);
        assert_eq!(route_intent("怎么用"), Intent::Tech);
    }

    #[test]
    fn routes_price_intent() {
        assert_eq!(route_intent("能便宜点吗"), Intent::Price);
        assert_eq!(route_intent("100元可以吗"), Intent::Price);
        assert_eq!(route_intent("能少50吗"), Intent::Price);
        assert_eq!(route_intent("多少钱"), Intent::Price);
    }

    #[test]
    fn routes_no_reply_and_default() {
        assert_eq!(route_intent("谢谢"), Intent::NoReply);
        assert_eq!(route_intent("好的"), Intent::NoReply);
        assert_eq!(route_intent("在吗"), Intent::Default);
        assert_eq!(route_intent("你好"), Intent::Default);
    }
}

//! 卡券模型与来源优先级选择。
//!
//! 对齐 Python 版 `_auto_delivery` 的卡券匹配逻辑：
//! 按来源优先级（own 自有 → dock_l1 一级对接 → dock_l2 二级对接）分组，
//! 取第一个"恰好 1 张卡"的分组；同组多张或全组为空则无匹配。

use serde::{Deserialize, Serialize};

/// 卡券来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardSource {
    /// 自有卡券。
    Own,
    /// 一级对接。
    DockL1,
    /// 二级对接。
    DockL2,
}

impl CardSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            CardSource::Own => "own",
            CardSource::DockL1 => "dock_l1",
            CardSource::DockL2 => "dock_l2",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        match value {
            "dock_l1" => CardSource::DockL1,
            "dock_l2" => CardSource::DockL2,
            _ => CardSource::Own,
        }
    }
}

/// 来源优先级（own 最高）。
const SOURCE_PRIORITY: [CardSource; 3] = [CardSource::Own, CardSource::DockL1, CardSource::DockL2];

/// 卡券（业务层从存储加载）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    #[serde(default)]
    pub id: i64,
    /// 所属用户（管理视角）。
    #[serde(default)]
    pub owner_id: i64,
    /// 所属账号（管理视角）。
    #[serde(default)]
    pub account_id: String,
    pub name: String,
    /// text / data / api / image。
    pub card_type: String,
    #[serde(default = "default_source")]
    pub source: CardSource,
    /// 是否启用（自动发货可选）。
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 固定文字内容（text 类型）。
    #[serde(default)]
    pub text_content: String,
    /// 批量数据内容（data 类型，多行一条）。
    #[serde(default)]
    pub data_content: String,
    /// 图片 URL（image 类型）。
    #[serde(default)]
    pub image_url: String,
    /// 多图片 URL 列表（JSON 数组字符串）。
    #[serde(default)]
    pub image_urls: String,
    /// API 拉取配置（JSON 字符串）。
    #[serde(default)]
    pub api_config: String,
    /// 发货延时（秒）。
    #[serde(default)]
    pub delay_seconds: u32,
    /// 备注（追加到内容后）。
    #[serde(default)]
    pub description: String,
}

fn default_source() -> CardSource {
    CardSource::Own
}

fn default_true() -> bool {
    true
}

impl Card {
    /// 是否为对接卡券（card_only 模式下跳过）。
    pub fn is_dock(&self) -> bool {
        matches!(self.source, CardSource::DockL1 | CardSource::DockL2)
    }
}

/// 卡券选择器 — 来源优先级唯一匹配。
pub struct CardSelector;

impl CardSelector {
    /// 按来源优先级选取唯一卡券。
    ///
    /// 返回 `(卡券, 来源)`；无唯一匹配返回 `None`。
    pub fn select(cards: &[Card]) -> Option<&Card> {
        SOURCE_PRIORITY
            .iter()
            .filter_map(|source| {
                let group: Vec<&Card> =
                    cards.iter().filter(|card| card.source == *source).collect();
                match group.len() {
                    1 => Some(group[0]),
                    _ => None, // 0 张或 >1 张均跳过该来源
                }
            })
            .next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(source: CardSource) -> Card {
        Card {
            id: 1,
            owner_id: 0,
            account_id: String::new(),
            name: "卡".to_string(),
            card_type: "text".to_string(),
            source,
            enabled: true,
            text_content: "内容".to_string(),
            data_content: String::new(),
            image_url: String::new(),
            image_urls: String::new(),
            api_config: String::new(),
            delay_seconds: 0,
            description: String::new(),
        }
    }

    #[test]
    fn selects_own_first() {
        let cards = vec![card(CardSource::DockL1), card(CardSource::Own)];
        assert_eq!(
            CardSelector::select(&cards).map(|c| c.source),
            Some(CardSource::Own)
        );
    }

    #[test]
    fn skips_source_with_multiple_cards() {
        // own 有 2 张 → 跳过，落到唯一 1 张的 dock_l1。
        let cards = vec![
            card(CardSource::Own),
            card(CardSource::Own),
            card(CardSource::DockL1),
        ];
        assert_eq!(
            CardSelector::select(&cards).map(|c| c.source),
            Some(CardSource::DockL1)
        );
    }

    #[test]
    fn none_when_no_unique_group() {
        let cards = vec![card(CardSource::Own), card(CardSource::Own)];
        assert!(CardSelector::select(&cards).is_none());
        assert!(CardSelector::select(&[]).is_none());
    }

    #[test]
    fn dock_detection() {
        assert!(card(CardSource::DockL1).is_dock());
        assert!(card(CardSource::DockL2).is_dock());
        assert!(!card(CardSource::Own).is_dock());
    }

    #[test]
    fn source_str_roundtrip() {
        assert_eq!(CardSource::from_str("dock_l1"), CardSource::DockL1);
        assert_eq!(CardSource::from_str("unknown"), CardSource::Own);
        assert_eq!(CardSource::Own.as_str(), "own");
    }
}

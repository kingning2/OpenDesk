//! 后端专属的多语言文案。
//!
//! 负责：
//! - 解析 IPC / UI 传入的 locale
//! - 按语言生成后端下发到前端的用户可见文案（直接下发译文，不传 key）
//!
//! 文案资源用 Fluent `.ftl` 文件（编译期经 `include_str!` 嵌入），
//! 目录划分：
//! - `mod.rs` — [`Locale`] 枚举与解析、[`Translator`]（Fluent 加载与格式化）
//! - `job.rs` — 采集任务状态 / 进度 / 失败文案
//! - `keywords.rs` — 关键词批次文案

pub mod job;
pub mod keywords;

pub use job::{
    cancelled, failed, keyword_done, prepare_keywords, progress, queued, status_unavailable,
    stop_message,
};
pub use keywords::{empty_batch, need_batch};

use fluent_bundle::concurrent::FluentBundle as ConcurrentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use std::sync::OnceLock;
use unic_langid::langid;

/// 后端用户可见语言。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    /// 简体中文。
    ZhCn,
    /// 美式英语。
    EnUs,
}

impl Locale {
    /// 从 IPC / UI 语言标签解析；未知或空则默认中文。
    ///
    /// # 参数
    ///
    /// * `raw` - 如 `zh-CN`、`en-US`、`en`
    ///
    /// # 返回值
    ///
    /// 解析后的 [`Locale`]。
    pub fn parse(raw: Option<&str>) -> Self {
        let Some(value) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
            return Self::ZhCn;
        };
        let lower = value.to_ascii_lowercase();
        if lower.starts_with("en") {
            Self::EnUs
        } else {
            Self::ZhCn
        }
    }
}

/// Fluent 文案格式化器。
///
/// 当前仅嵌入 `zh-CN.ftl`；[`Locale::EnUs`] 暂回退中文，后续补 `en-US.ftl` 后
/// 在此按语言加载对应 bundle。
pub struct Translator {
    zh: ConcurrentBundle<FluentResource>,
}

impl Translator {
    fn new() -> Self {
        let mut zh = ConcurrentBundle::new_concurrent(vec![langid!("zh-CN")]);
        zh.set_use_isolating(false);
        let resource =
            FluentResource::try_new(include_str!("../../assets/i18n/zh-CN.ftl").to_string())
                .expect("embedded zh-CN.ftl must parse");
        zh.add_resource(resource)
            .expect("embedded zh-CN.ftl must have no errors");
        Self { zh }
    }

    /// 按中文文案格式化 message。
    ///
    /// 缺失的 message 或 value 返回原 id。
    pub fn t(&self, id: &str, args: &[(&str, FluentValue<'_>)]) -> String {
        let Some(message) = self.zh.get_message(id) else {
            return id.to_string();
        };
        let Some(value) = message.value() else {
            return id.to_string();
        };
        let mut errors = Vec::new();
        let mut fluent_args = FluentArgs::new();
        for (key, value) in args {
            fluent_args.set(*key, (*value).clone());
        }
        let rendered = self
            .zh
            .format_pattern(value, Some(&fluent_args), &mut errors)
            .into_owned();
        for error in errors {
            tracing::warn!(target: "common::i18n", message_id = id, error = %error, "fluent format error");
        }
        rendered
    }
}

static TRANSLATOR: OnceLock<Translator> = OnceLock::new();

/// 访问全局中文文案翻译器。
pub fn translator() -> &'static Translator {
    TRANSLATOR.get_or_init(Translator::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_defaults_to_chinese() {
        assert_eq!(Locale::parse(None), Locale::ZhCn);
        assert_eq!(Locale::parse(Some("")), Locale::ZhCn);
        assert_eq!(Locale::parse(Some("  ")), Locale::ZhCn);
        assert_eq!(Locale::parse(Some("unknown")), Locale::ZhCn);
        assert_eq!(Locale::parse(Some("ja-JP")), Locale::ZhCn);
    }

    #[test]
    fn parse_en_prefix_matches_english() {
        assert_eq!(Locale::parse(Some("en-US")), Locale::EnUs);
        assert_eq!(Locale::parse(Some("en")), Locale::EnUs);
        assert_eq!(Locale::parse(Some("EN-GB")), Locale::EnUs);
    }

    #[test]
    fn formats_job_messages() {
        let translator = translator();
        assert_eq!(translator.t("job-queued", &[]), "排队中");
        assert_eq!(
            translator.t("job-prepare-keywords", &[("keywords_total", 3.into())]),
            "准备爬取 3 个关键词"
        );
        assert_eq!(
            translator.t(
                "job-progress",
                &[
                    ("done", 1.into()),
                    ("total", 5.into()),
                    ("keyword", "youtube".into()),
                    ("keyword_accepted", 2.into()),
                    ("accepted_count", 7.into()),
                ]
            ),
            "关键词进度 1/5 · 当前「youtube」· 本词收录 2 · 合计收录 7"
        );
        assert_eq!(
            translator.t("job-failed", &[("error", "boom".into())]),
            "失败：boom"
        );
        assert_eq!(
            translator.t("stop-quota-exceeded", &[]),
            "YouTube 配额已用尽，已自动停止爬虫"
        );
        assert_eq!(
            translator.t("keyword-empty-batch", &[("batch", "b1".into())]),
            "批次 b1 没有可用关键词"
        );
    }

    #[test]
    fn missing_message_returns_id() {
        assert_eq!(translator().t("no-such-message", &[]), "no-such-message");
    }
}

//! Crawler 用户可见文案的后端多语言。
//!
//! 负责：
//! - 解析前端传入的 locale
//! - 按语言生成 job status / 错误提示字符串（IPC 直接下发译文，不传 key）
//!
//! 作者：coisini
//! 创建时间：2026-07-20

/// 采集任务用户可见语言。
///
/// 作者：coisini
/// 创建时间：2026-07-20
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
    /// 作者：coisini
    /// 创建时间：2026-07-20
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

/// 排队中。
///
/// 作者：coisini
/// 创建时间：2026-07-20
pub fn queued(locale: Locale) -> String {
    match locale {
        Locale::ZhCn => "排队中".to_string(),
        Locale::EnUs => "Queued".to_string(),
    }
}

/// 准备爬取 N 个关键词。
///
/// 作者：coisini
/// 创建时间：2026-07-20
pub fn prepare_keywords(locale: Locale, keywords_total: usize) -> String {
    match locale {
        Locale::ZhCn => format!("准备爬取 {keywords_total} 个关键词"),
        Locale::EnUs => format!("Preparing to crawl {keywords_total} keywords"),
    }
}

/// 任务已取消。
///
/// 作者：coisini
/// 创建时间：2026-07-20
pub fn cancelled(locale: Locale) -> String {
    match locale {
        Locale::ZhCn => "任务已取消".to_string(),
        Locale::EnUs => "Job cancelled".to_string(),
    }
}

/// 关键词进行中进度。
///
/// 作者：coisini
/// 创建时间：2026-07-20
pub fn progress(
    locale: Locale,
    done: i64,
    total: i64,
    keyword: &str,
    keyword_accepted: i64,
    accepted_count: i64,
) -> String {
    match locale {
        Locale::ZhCn => format!(
            "关键词进度 {done}/{total} · 当前「{keyword}」· 本词收录 {keyword_accepted} · 合计收录 {accepted_count}"
        ),
        Locale::EnUs => format!(
            "Keyword progress {done}/{total} · current “{keyword}” · accepted this keyword {keyword_accepted} · total accepted {accepted_count}"
        ),
    }
}

/// 单个关键词完成进度。
///
/// 作者：coisini
/// 创建时间：2026-07-20
pub fn keyword_done(locale: Locale, done: i64, total: i64) -> String {
    match locale {
        Locale::ZhCn => format!("关键词进度 {done}/{total}"),
        Locale::EnUs => format!("Keyword progress {done}/{total}"),
    }
}

/// 失败摘要（含错误细节）。
///
/// 作者：coisini
/// 创建时间：2026-07-20
pub fn failed(locale: Locale, error: &str) -> String {
    match locale {
        Locale::ZhCn => format!("失败：{error}"),
        Locale::EnUs => format!("Failed: {error}"),
    }
}

/// 状态锁不可用。
///
/// 作者：coisini
/// 创建时间：2026-07-20
pub fn status_unavailable(locale: Locale) -> String {
    match locale {
        Locale::ZhCn => "状态不可用".to_string(),
        Locale::EnUs => "Status unavailable".to_string(),
    }
}

/// 按 stop_reason 生成结束文案。
///
/// 作者：coisini
/// 创建时间：2026-07-20
pub fn stop_message(locale: Locale, stop_reason: &str) -> String {
    match (locale, stop_reason) {
        (Locale::ZhCn, "keywords_finished") => "已完成".to_string(),
        (Locale::ZhCn, "max_total_reached") => "已达数量上限".to_string(),
        (Locale::ZhCn, "quota_exceeded") => "YouTube 配额已用尽，已自动停止爬虫".to_string(),
        (Locale::ZhCn, "cancelled") => cancelled(Locale::ZhCn),
        (Locale::ZhCn, _) => "已结束".to_string(),
        (Locale::EnUs, "keywords_finished") => "Completed".to_string(),
        (Locale::EnUs, "max_total_reached") => "Reached max result limit".to_string(),
        (Locale::EnUs, "quota_exceeded") => {
            "YouTube quota exhausted; crawl stopped automatically".to_string()
        }
        (Locale::EnUs, "cancelled") => cancelled(Locale::EnUs),
        (Locale::EnUs, _) => "Ended".to_string(),
    }
}

/// 未选择关键词批次。
///
/// 作者：coisini
/// 创建时间：2026-07-20
pub fn need_batch(locale: Locale) -> String {
    match locale {
        Locale::ZhCn => "请先导入 CSV 并选择关键词批次".to_string(),
        Locale::EnUs => "Import a CSV and select a keyword batch first".to_string(),
    }
}

/// 批次无可用关键词。
///
/// 作者：coisini
/// 创建时间：2026-07-20
pub fn empty_batch(locale: Locale, batch: &str) -> String {
    match locale {
        Locale::ZhCn => format!("批次 {batch} 没有可用关键词"),
        Locale::EnUs => format!("Batch {batch} has no usable keywords"),
    }
}

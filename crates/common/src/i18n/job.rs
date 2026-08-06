//! 采集任务状态、进度与失败的用户可见文案。

use super::{translator, Locale};
use fluent_bundle::FluentValue;

fn t(id: &str, args: &[(&str, FluentValue<'_>)]) -> String {
    translator().t(id, args)
}

/// 排队中。
pub fn queued(_locale: Locale) -> String {
    t("job-queued", &[])
}

/// 准备爬取 N 个关键词。
pub fn prepare_keywords(_locale: Locale, keywords_total: usize) -> String {
    t(
        "job-prepare-keywords",
        &[("keywords_total", (keywords_total as i64).into())],
    )
}

/// 任务已取消。
pub fn cancelled(_locale: Locale) -> String {
    t("job-cancelled", &[])
}

/// 关键词进行中进度。
pub fn progress(
    _locale: Locale,
    done: i64,
    total: i64,
    keyword: &str,
    keyword_accepted: i64,
    accepted_count: i64,
) -> String {
    t(
        "job-progress",
        &[
            ("done", done.into()),
            ("total", total.into()),
            ("keyword", keyword.into()),
            ("keyword_accepted", keyword_accepted.into()),
            ("accepted_count", accepted_count.into()),
        ],
    )
}

/// 单个关键词完成进度。
pub fn keyword_done(_locale: Locale, done: i64, total: i64) -> String {
    t(
        "job-keyword-done",
        &[("done", done.into()), ("total", total.into())],
    )
}

/// 失败摘要（含错误细节）。
pub fn failed(_locale: Locale, error: &str) -> String {
    t("job-failed", &[("error", error.into())])
}

/// 状态锁不可用。
pub fn status_unavailable(_locale: Locale) -> String {
    t("job-status-unavailable", &[])
}

/// 按 stop_reason 生成结束文案。
///
/// 当前仅中文 bundle；[`Locale::EnUs`] 一并回退中文。
pub fn stop_message(_locale: Locale, stop_reason: &str) -> String {
    let id = match stop_reason {
        "keywords_finished" => "stop-keywords-finished",
        "max_total_reached" => "stop-max-total-reached",
        "quota_exceeded" => "stop-quota-exceeded",
        "cancelled" => "stop-cancelled",
        _ => "stop-other",
    };
    t(id, &[])
}

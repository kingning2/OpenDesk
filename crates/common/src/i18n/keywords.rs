//! 关键词批次的用户可见文案。

use super::{translator, Locale};
use fluent_bundle::FluentValue;

fn t(id: &str, args: &[(&str, FluentValue<'_>)]) -> String {
    translator().t(id, args)
}

/// 未选择关键词批次。
pub fn need_batch(_locale: Locale) -> String {
    t("keyword-need-batch", &[])
}

/// 批次无可用关键词。
pub fn empty_batch(_locale: Locale, batch: &str) -> String {
    t("keyword-empty-batch", &[("batch", batch.into())])
}

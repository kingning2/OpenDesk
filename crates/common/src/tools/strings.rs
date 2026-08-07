//! 字符串工具：消除各 crate 内重复的 CSV 切分、邮箱归一化等。

/// 将可选逗号分隔字符串切分为非空、trim 后的列表。
pub fn split_csv(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_csv_trims_and_filters_empty() {
        assert_eq!(split_csv(Some(" a , b,, c ")), vec!["a", "b", "c"]);
        assert!(split_csv(None).is_empty());
        assert!(split_csv(Some("")).is_empty());
    }
}

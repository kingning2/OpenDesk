//! 存储层字符串工具。

/// 归一化邮箱：trim + 小写。
pub fn normalize_email(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_email_trims_and_lowercases() {
        assert_eq!(normalize_email("  Foo@Example.COM "), "foo@example.com");
    }
}

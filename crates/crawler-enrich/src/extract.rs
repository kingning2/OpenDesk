//! Best-effort email extraction shared with stage-1 API crawler rules.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-20

/// Extract the first plausible email token from free text.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-20
pub fn extract_email(text: &str) -> Option<String> {
    let normalized = text
        .replace("[at]", "@")
        .replace("(at)", "@")
        .replace("[dot]", ".")
        .replace("(dot)", ".");
    let mut token = String::new();
    for ch in normalized.chars() {
        if ch.is_whitespace() {
            if token.contains('@') && token.contains('.') && token.len() >= 5 {
                return Some(token);
            }
            token.clear();
        } else {
            token.push(ch);
        }
    }
    if token.contains('@') && token.contains('.') && token.len() >= 5 {
        Some(token)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_plain_email() {
        assert_eq!(
            extract_email("reach us at team@example.com today"),
            Some("team@example.com".to_string())
        );
    }

    #[test]
    fn normalizes_obfuscated_email() {
        assert_eq!(
            extract_email("mail team[at]example[dot]com"),
            Some("team@example.com".to_string())
        );
    }
}

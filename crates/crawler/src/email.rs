//! Best-effort email extraction shared by API 采集（youtube）与 RPA 补全（enrich）。

/// Extract the first plausible email token from free text.
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

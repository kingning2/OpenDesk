//! 闲鱼 Cookie 解析。

use std::collections::HashMap;

/// 解析浏览器导出的 `COOKIES_STR`（`k=v; k2=v2; ...`）为键值表。
pub fn parse_cookies(cookie_str: &str) -> HashMap<String, String> {
    cookie_str
        .split(';')
        .filter_map(|part| {
            let part = part.trim();
            if part.is_empty() {
                return None;
            }
            let (key, value) = part.split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

/// 取用户 id（`unb` cookie）。
pub fn my_id(cookies: &HashMap<String, String>) -> Option<String> {
    cookies.get("unb").cloned()
}

/// 取签名 token（`_m_h5_tk` 的 `_` 前缀）。
pub fn sign_token(cookies: &HashMap<String, String>) -> Option<String> {
    cookies
        .get("_m_h5_tk")
        .map(|value| value.split('_').next().unwrap_or_default().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_standard_cookie_string() {
        let cookies =
            parse_cookies("unb=U-123; _m_h5_tk=abc_token_987; cookie2=xyz; cna=cnaval");
        assert_eq!(cookies.get("unb"), Some(&"U-123".to_string()));
        assert_eq!(cookies.get("_m_h5_tk"), Some(&"abc_token_987".to_string()));
    }

    #[test]
    fn parse_handles_spaces_and_missing() {
        let cookies = parse_cookies("  a=1 ; b=2");
        assert_eq!(cookies.len(), 2);
        assert_eq!(cookies.get("a"), Some(&"1".to_string()));
        let empty = parse_cookies("");
        assert!(empty.is_empty());
    }

    #[test]
    fn my_id_and_sign_token_extraction() {
        let cookies = parse_cookies("unb=U-9; _m_h5_tk=tokenpart_rest");
        assert_eq!(my_id(&cookies), Some("U-9".to_string()));
        assert_eq!(sign_token(&cookies), Some("tokenpart".to_string()));
    }
}

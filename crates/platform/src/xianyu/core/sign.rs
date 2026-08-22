//! 闲鱼接口签名 — MD5 摘要。

use md5::{Digest, Md5};

/// 闲鱼 H5 接口固定 appKey。
pub const APP_KEY: &str = "34839810";
/// WebSocket /reg 注册用的钉钉 appKey。
pub const REG_APP_KEY: &str = "444e9908a51d1cb236a27862abc769c9";

/// 生成接口签名：`md5("{token}&{t}&{app_key}&{data}")`。
///
/// `token` 取 Cookie `_m_h5_tk` 的 `_` 前缀部分；`t` 为毫秒时间戳。
pub fn generate_sign(token: &str, t: &str, data: &str) -> String {
    let msg = format!("{token}&{t}&{APP_KEY}&{data}");
    let mut hasher = Md5::new();
    hasher.update(msg.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_is_stable_md5() {
        // 固定输入，断言输出为 32 位十六进制 MD5。
        let sign = generate_sign("tk-token-123", "1700000000000", r#"{"appKey":"abc"}"#);
        assert_eq!(sign.len(), 32);
        assert!(sign.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn sign_changes_with_data() {
        let a = generate_sign("t1", "1000", "data1");
        let b = generate_sign("t1", "1000", "data2");
        assert_ne!(a, b);
    }
}

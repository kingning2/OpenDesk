//! 激活 token 签发（OpenSSL RSA-PSS）。
//!
//! 作者：coisini
//! 创建时间：2026-07-16

use std::path::Path;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{DateTime, NaiveDate};

use crate::claims::{activation_code_from_claims, now_ts, signing_message, LicenseClaims};
use crate::crypto::OpenSslRsaPss;

/// 过期策略：绝对时间或首次激活起算的时长。
///
/// 作者：coisini
/// 创建时间：2026-07-16
#[derive(Debug, Clone)]
pub enum ExpiryPolicy {
    /// 绝对过期 Unix 秒（`--exp`）。
    Absolute {
        /// 截止时间。
        exp: i64,
    },
    /// 有效秒数，从本机首次激活起算（`--days`）。
    DurationFromActivation {
        /// 有效秒数。
        duration_secs: i64,
    },
}

/// 将过期参数解析为 Unix 秒。
///
/// 作者：coisini
/// 创建时间：2026-07-16
pub fn parse_exp_to_unix_seconds(exp: &str) -> Result<i64, String> {
    let s = exp.trim();
    if s.chars().all(|c| c.is_ascii_digit()) {
        return s
            .parse::<i64>()
            .map_err(|e| format!("invalid exp seconds: {e}"));
    }

    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.timestamp());
    }
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return date
            .and_hms_opt(0, 0, 0)
            .map(|v| v.and_utc().timestamp())
            .ok_or_else(|| "invalid date".to_string());
    }
    Err("unsupported exp format, use unix seconds / RFC3339 / YYYY-MM-DD".to_string())
}

/// 解析 `--exp` 或 `--days` 为过期策略。
///
/// `--days` → 首次激活起算；`--exp` → 绝对截止。
///
/// 作者：coisini
/// 创建时间：2026-07-16
pub fn resolve_expiry_policy(
    exp_arg: Option<String>,
    days_arg: Option<i64>,
) -> Result<ExpiryPolicy, String> {
    if exp_arg.is_some() && days_arg.is_some() {
        return Err("use either --exp or --days, not both".to_string());
    }

    if let Some(exp_text) = exp_arg {
        return Ok(ExpiryPolicy::Absolute {
            exp: parse_exp_to_unix_seconds(&exp_text)?,
        });
    }
    if let Some(days) = days_arg {
        if days <= 0 {
            return Err("--days must be > 0".to_string());
        }
        return Ok(ExpiryPolicy::DurationFromActivation {
            duration_secs: days * 24 * 60 * 60,
        });
    }
    Err("missing expiration: provide --exp or --days".to_string())
}

/// 兼容旧调用：仅解析绝对 `exp`（由 policy 推导展示用截止点）。
///
/// 作者：coisini
/// 创建时间：2026-07-16
#[deprecated(note = "use resolve_expiry_policy")]
pub fn resolve_exp(exp_arg: Option<String>, days_arg: Option<i64>) -> Result<i64, String> {
    match resolve_expiry_policy(exp_arg, days_arg)? {
        ExpiryPolicy::Absolute { exp } => Ok(exp),
        ExpiryPolicy::DurationFromActivation { duration_secs } => Ok(now_ts() + duration_secs),
    }
}

/// 解析私钥路径（默认 `./keys/private_key.pem`）。
///
/// 作者：coisini
/// 创建时间：2026-07-16
pub fn resolve_private_key_path(private_key: Option<String>) -> Result<String, String> {
    if let Some(path) = private_key {
        return Ok(path);
    }

    let default_path = std::env::current_dir()
        .map_err(|e| format!("failed to get current dir: {e}"))?
        .join("keys")
        .join("private_key.pem");

    if !default_path.is_file() {
        return Err(format!(
            "private key file not found: {}",
            default_path.display()
        ));
    }

    Ok(default_path.to_string_lossy().to_string())
}

fn sign_rsa_pss_sha256(message: &str, private_key_path: &str) -> Result<Vec<u8>, String> {
    if !Path::new(private_key_path).is_file() {
        return Err(format!("private key file not found: {private_key_path}"));
    }
    let pem =
        std::fs::read_to_string(private_key_path).map_err(|e| format!("read key failed: {e}"))?;
    OpenSslRsaPss::new().sign(message.as_bytes(), &pem)
}

/// 按策略生成激活 token。
///
/// 作者：coisini
/// 创建时间：2026-07-16
pub fn generate_activation_token_with_policy(
    machine_code: String,
    policy: ExpiryPolicy,
    product: String,
    version: String,
    private_key_path: String,
) -> Result<String, String> {
    let iat = now_ts();
    let (exp, duration_secs) = match policy {
        ExpiryPolicy::Absolute { exp } => (exp, None),
        ExpiryPolicy::DurationFromActivation { duration_secs } => {
            // exp 仅作「若立即激活」的参考展示；真正计时在首次激活。
            (iat + duration_secs, Some(duration_secs))
        }
    };

    let message = signing_message(&product, &version, &machine_code, exp, duration_secs);
    let sig = sign_rsa_pss_sha256(&message, &private_key_path)?;
    let sig_b64 = URL_SAFE_NO_PAD.encode(sig);

    let claims = LicenseClaims {
        product,
        v: version,
        machine_code,
        exp,
        iat,
        duration_secs,
        sig: sig_b64,
    };
    activation_code_from_claims(&claims)
}

/// 生成绝对过期 token（兼容旧签名）。
///
/// 作者：coisini
/// 创建时间：2026-07-16
pub fn generate_activation_token(
    machine_code: String,
    exp: i64,
    product: String,
    version: String,
    private_key_path: String,
) -> Result<String, String> {
    generate_activation_token_with_policy(
        machine_code,
        ExpiryPolicy::Absolute { exp },
        product,
        version,
        private_key_path,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claims::parse_activation_code;
    use crate::crypto::OpenSslRsaPss;
    use crate::embedded::EmbeddedMaterials;
    use std::path::PathBuf;

    fn keys_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("keys")
    }

    #[test]
    fn openssl_sign_verify_roundtrip() {
        let private_path = keys_dir().join("private_key.pem");
        if !private_path.is_file() {
            eprintln!("skip: private_key.pem missing");
            return;
        }
        let pem = std::fs::read_to_string(&private_path).expect("read private");
        let public_pem = EmbeddedMaterials::new()
            .public_key_pem()
            .expect("public pem");
        let message = b"opendesk|1|machine|123";
        let crypto = OpenSslRsaPss::new();
        let sig = crypto.sign(message, &pem).expect("sign");
        crypto.verify(message, &sig, &public_pem).expect("verify");
    }

    #[test]
    fn days_token_uses_duration_secs_in_signature() {
        let private_path = keys_dir().join("private_key.pem");
        if !private_path.is_file() {
            eprintln!("skip: private_key.pem missing");
            return;
        }
        let token = generate_activation_token_with_policy(
            "abc".into(),
            ExpiryPolicy::DurationFromActivation {
                duration_secs: 3 * 86400,
            },
            "opendesk".into(),
            "1".into(),
            private_path.to_string_lossy().into(),
        )
        .expect("token");
        let claims = parse_activation_code(&token).expect("parse");
        assert_eq!(claims.duration_secs, Some(3 * 86400));
        let message = signing_message(
            &claims.product,
            &claims.v,
            &claims.machine_code,
            claims.exp,
            claims.duration_secs,
        );
        let sig = URL_SAFE_NO_PAD.decode(claims.sig.trim()).expect("sig b64");
        let public_pem = EmbeddedMaterials::new().public_key_pem().expect("public");
        OpenSslRsaPss::new()
            .verify(message.as_bytes(), &sig, &public_pem)
            .expect("verify token");
    }
}

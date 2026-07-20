//! 本地 license 校验（OpenSSL + 机器码 + 过期）。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-16

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::Serialize;

use crate::activation_state::ActivationStateStore;
use crate::attestation::AttestationService;
use crate::claims::{now_ts, parse_activation_code, signing_message};
use crate::crypto::OpenSslRsaPss;
use crate::embedded::EmbeddedMaterials;
use crate::key_file::resolve_token_input;
use crate::machine_code::compute_machine_code;

/// 校验输出 JSON（主程序解析）。
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyOutput {
    /// 是否整体通过。
    pub valid: bool,
    /// 失败原因。
    pub reason: Option<String>,
    /// 产品名。
    pub product: String,
    /// 版本。
    pub version: String,
    /// Token 内机器码。
    pub token_machine_code: String,
    /// 本机机器码。
    pub local_machine_code: String,
    /// 机器码是否匹配。
    pub machine_match: bool,
    /// 是否未过期。
    pub token_valid_time: bool,
    /// 有效期截止 Unix 秒（时长模式为 activated_at + duration）。
    pub token_expired_at: i64,
    /// 当前时间。
    pub now: i64,
    /// 签名是否通过。
    pub signature_verified: bool,
    /// 回显的 nonce（可空）。
    pub nonce: Option<String>,
    /// HMAC attestation（有 nonce 时必填）。
    pub attestation: Option<String>,
    /// 首次激活时间（时长模式）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activated_at: Option<i64>,
}

fn verify_signature(claims: &crate::claims::LicenseClaims) -> Result<(), String> {
    let message = signing_message(
        &claims.product,
        &claims.v,
        &claims.machine_code,
        claims.exp,
        claims.duration_secs,
    );
    let sig_bytes = URL_SAFE_NO_PAD
        .decode(claims.sig.trim())
        .map_err(|e| format!("sig base64url decode failed: {e}"))?;
    let public_pem = EmbeddedMaterials::new().public_key_pem()?;
    OpenSslRsaPss::new().verify(message.as_bytes(), &sig_bytes, &public_pem)
}

/// 解析有效期：时长模式从首次激活起算，否则用绝对 exp。
fn resolve_effective_expiry(
    token: &str,
    claims: &crate::claims::LicenseClaims,
    now: i64,
    state_dir: Option<&str>,
) -> Result<(i64, Option<i64>, bool), String> {
    if let Some(duration_secs) = claims.duration_secs {
        let dir = state_dir.ok_or_else(|| {
            "duration-based license requires --state-dir (OpenDesk passes app data dir)".to_string()
        })?;
        let store = ActivationStateStore::new(dir);
        let (activated_at, expires_at) = store.resolve_expiry(token, duration_secs, now)?;
        let valid_time = now <= expires_at;
        Ok((expires_at, Some(activated_at), valid_time))
    } else {
        Ok((claims.exp, None, now <= claims.exp))
    }
}

/// 执行本地校验，可选 nonce 挑战应答。
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
///
/// # 参数
///
/// * `token` / `key_file` - 二选一
/// * `now` - 可选覆盖当前时间（测试用）
/// * `nonce` - 主程序挑战值；非空时输出 attestation
/// * `state_dir` - 时长 license 的激活状态目录
pub fn verify_local(
    token: Option<String>,
    key_file: Option<String>,
    now: Option<i64>,
    nonce: Option<String>,
    state_dir: Option<String>,
) -> Result<(VerifyOutput, i32), String> {
    let token = resolve_token_input(token, key_file)?;
    let claims = parse_activation_code(&token)?;
    let local_machine_code = compute_machine_code()?;
    let now = now.unwrap_or_else(now_ts);

    let machine_match = claims.machine_code == local_machine_code;
    let signature_result = verify_signature(&claims);
    let signature_verified = signature_result.is_ok();

    // 仅在签名与机器码通过后再落盘首次激活时间，避免无效 token 占坑。
    let (token_expired_at, activated_at, token_valid_time) = if signature_verified && machine_match
    {
        resolve_effective_expiry(&token, &claims, now, state_dir.as_deref())?
    } else if let Some(duration) = claims.duration_secs {
        (claims.iat.saturating_add(duration), None, false)
    } else {
        (claims.exp, None, now <= claims.exp)
    };

    let (valid, reason) = if !signature_verified {
        (
            false,
            Some(
                signature_result
                    .err()
                    .unwrap_or_else(|| "signature verify failed".to_string()),
            ),
        )
    } else if !machine_match {
        (false, Some("machineCode mismatch".to_string()))
    } else if !token_valid_time {
        (false, Some("license expired".to_string()))
    } else {
        (true, None)
    };

    let nonce_value = nonce
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let attestation = if let Some(ref nonce_ref) = nonce_value {
        let service = AttestationService::new(EmbeddedMaterials::new().attestation_key());
        Some(service.sign(
            nonce_ref,
            valid,
            &claims.product,
            token_expired_at,
            &local_machine_code,
        )?)
    } else {
        None
    };

    let output = VerifyOutput {
        valid,
        reason,
        product: claims.product.clone(),
        version: claims.v.clone(),
        token_machine_code: claims.machine_code.clone(),
        local_machine_code,
        machine_match,
        token_valid_time,
        token_expired_at,
        now,
        signature_verified,
        nonce: nonce_value,
        attestation,
        activated_at,
    };

    Ok((output, if valid { 0 } else { 1 }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sign::{generate_activation_token_with_policy, ExpiryPolicy};
    use std::path::PathBuf;

    fn keys_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("keys")
    }

    #[test]
    fn duration_license_starts_clock_on_first_verify() {
        let private_path = keys_dir().join("private_key.pem");
        if !private_path.is_file() {
            eprintln!("skip: private_key.pem missing");
            return;
        }

        // 绕过真机码：直接测 ActivationStateStore + claims 逻辑用 store。
        let dir = std::env::temp_dir().join(format!("opendesk-act-state-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");

        let token = generate_activation_token_with_policy(
            "test-machine".into(),
            ExpiryPolicy::DurationFromActivation { duration_secs: 10 },
            "opendesk".into(),
            "1".into(),
            private_path.to_string_lossy().into(),
        )
        .expect("token");

        let store = ActivationStateStore::new(&dir);
        let t0 = 1_700_000_000_i64;
        let (activated, exp1) = store.resolve_expiry(&token, 10, t0).expect("first");
        assert_eq!(activated, t0);
        assert_eq!(exp1, t0 + 10);

        let (activated2, exp2) = store.resolve_expiry(&token, 10, t0 + 5).expect("second");
        assert_eq!(activated2, t0);
        assert_eq!(exp2, t0 + 10);

        let _ = std::fs::remove_dir_all(&dir);
    }
}

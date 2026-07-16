//! Verifier 挑战应答（HMAC-SHA256）。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-16

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Attestation 计算器。
///
/// 功能：
///
/// - 根据 nonce 与校验结果字段生成/校验 HMAC
/// - 防止「永远返回 valid」的伪造 verifier 轻易替换
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
pub struct AttestationService {
    key: Vec<u8>,
}

impl AttestationService {
    /// 使用原始密钥构造服务。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-16
    ///
    /// # 参数
    ///
    /// * `key` - HMAC 密钥字节
    ///
    /// # 返回值
    ///
    /// 返回 [`AttestationService`]。
    pub fn new(key: &[u8]) -> Self {
        Self { key: key.to_vec() }
    }

    /// 计算 attestation 十六进制字符串。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-16
    ///
    /// # 参数
    ///
    /// * `nonce` - 主程序下发的随机 nonce（hex 或任意非空串）
    /// * `valid` - 校验是否通过
    /// * `product` - 产品名
    /// * `token_expired_at` - 过期 Unix 秒
    /// * `local_machine_code` - 本机机器码
    ///
    /// # 返回值
    ///
    /// 小写 hex HMAC-SHA256。
    pub fn sign(
        &self,
        nonce: &str,
        valid: bool,
        product: &str,
        token_expired_at: i64,
        local_machine_code: &str,
    ) -> Result<String, String> {
        let payload = Self::payload(nonce, valid, product, token_expired_at, local_machine_code);
        let mut mac = HmacSha256::new_from_slice(&self.key)
            .map_err(|error| format!("HMAC key init failed: {error}"))?;
        mac.update(payload.as_bytes());
        let bytes = mac.finalize().into_bytes();
        Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
    }

    /// 校验 attestation 是否匹配。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-16
    ///
    /// # 参数
    ///
    /// * `expected_hex` - verifier 返回的 attestation
    /// * 其余参数与 [`Self::sign`] 相同
    ///
    /// # 返回值
    ///
    /// 匹配返回 `Ok(())`；不匹配返回错误。
    pub fn verify(
        &self,
        expected_hex: &str,
        nonce: &str,
        valid: bool,
        product: &str,
        token_expired_at: i64,
        local_machine_code: &str,
    ) -> Result<(), String> {
        let actual = self.sign(nonce, valid, product, token_expired_at, local_machine_code)?;
        if actual.eq_ignore_ascii_case(expected_hex.trim()) {
            Ok(())
        } else {
            Err(
                "license attestation mismatch: verifier response failed challenge-response. \
                 Rebuild license-verifier with matching attestation key and do not replace the binary"
                    .into(),
            )
        }
    }

    fn payload(
        nonce: &str,
        valid: bool,
        product: &str,
        token_expired_at: i64,
        local_machine_code: &str,
    ) -> String {
        format!("{nonce}|{valid}|{product}|{token_expired_at}|{local_machine_code}")
    }
}

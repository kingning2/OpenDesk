//! OpenSSL（vendored）RSA-PSS + SHA-256 签验封装。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-16

use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private, Public};
use openssl::rsa::Padding;
use openssl::sign::{RsaPssSaltlen, Signer, Verifier};

/// OpenSSL 密码学服务。
///
/// 功能：
///
/// - 使用静态链接的 OpenSSL 做 RSA-PSS(SHA-256) 签名与验签
/// - 盐长度固定为 digest length，与历史 `rsa` crate 默认行为对齐
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
pub struct OpenSslRsaPss;

impl OpenSslRsaPss {
    /// 构造服务（无状态）。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-16
    ///
    /// # 返回值
    ///
    /// 返回 [`OpenSslRsaPss`] 实例。
    pub fn new() -> Self {
        Self
    }

    /// 使用 PKCS#8 PEM 私钥对消息做 RSA-PSS(SHA-256) 签名。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-16
    ///
    /// # 参数
    ///
    /// * `message` - 待签名明文字节
    /// * `private_key_pem` - PKCS#8 PEM 私钥全文
    ///
    /// # 返回值
    ///
    /// 成功返回签名字节；失败返回说明如何修复的错误字符串。
    pub fn sign(&self, message: &[u8], private_key_pem: &str) -> Result<Vec<u8>, String> {
        let key = PKey::private_key_from_pem(private_key_pem.as_bytes()).map_err(|error| {
            format!(
                "OpenSSL parse private key PEM failed: {error}. \
                 Ensure keys/private_key.pem is PKCS#8 PEM"
            )
        })?;
        self.sign_with_key(message, &key)
    }

    /// 使用 PKCS#8 / SPKI PEM 公钥验签。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-16
    ///
    /// # 参数
    ///
    /// * `message` - 原始消息
    /// * `signature` - PSS 签名字节
    /// * `public_key_pem` - 公钥 PEM
    ///
    /// # 返回值
    ///
    /// 验签成功返回 `Ok(())`；失败返回错误说明。
    pub fn verify(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key_pem: &str,
    ) -> Result<(), String> {
        let key = PKey::public_key_from_pem(public_key_pem.as_bytes()).map_err(|error| {
            format!(
                "OpenSSL parse public key PEM failed: {error}. \
                 Rebuild verifier after regenerating keys/public_key.pem"
            )
        })?;
        self.verify_with_key(message, signature, &key)
    }

    fn sign_with_key(&self, message: &[u8], key: &PKey<Private>) -> Result<Vec<u8>, String> {
        let mut signer = Signer::new(MessageDigest::sha256(), key)
            .map_err(|error| format!("OpenSSL create Signer failed: {error}"))?;
        signer
            .set_rsa_padding(Padding::PKCS1_PSS)
            .map_err(|error| format!("OpenSSL set PSS padding failed: {error}"))?;
        signer
            .set_rsa_mgf1_md(MessageDigest::sha256())
            .map_err(|error| format!("OpenSSL set MGF1 SHA-256 failed: {error}"))?;
        signer
            .set_rsa_pss_saltlen(RsaPssSaltlen::DIGEST_LENGTH)
            .map_err(|error| format!("OpenSSL set PSS salt length failed: {error}"))?;
        signer
            .update(message)
            .map_err(|error| format!("OpenSSL signer update failed: {error}"))?;
        signer
            .sign_to_vec()
            .map_err(|error| format!("OpenSSL sign failed: {error}"))
    }

    fn verify_with_key(
        &self,
        message: &[u8],
        signature: &[u8],
        key: &PKey<Public>,
    ) -> Result<(), String> {
        let mut verifier = Verifier::new(MessageDigest::sha256(), key)
            .map_err(|error| format!("OpenSSL create Verifier failed: {error}"))?;
        verifier
            .set_rsa_padding(Padding::PKCS1_PSS)
            .map_err(|error| format!("OpenSSL set PSS padding failed: {error}"))?;
        verifier
            .set_rsa_mgf1_md(MessageDigest::sha256())
            .map_err(|error| format!("OpenSSL set MGF1 SHA-256 failed: {error}"))?;
        verifier
            .set_rsa_pss_saltlen(RsaPssSaltlen::DIGEST_LENGTH)
            .map_err(|error| format!("OpenSSL set PSS salt length failed: {error}"))?;
        verifier
            .update(message)
            .map_err(|error| format!("OpenSSL verifier update failed: {error}"))?;
        let ok = verifier
            .verify(signature)
            .map_err(|error| format!("OpenSSL verify call failed: {error}"))?;
        if ok {
            Ok(())
        } else {
            Err("OpenSSL RSA-PSS signature mismatch. \
                 Re-issue the license with the matching private key"
                .into())
        }
    }
}

impl Default for OpenSslRsaPss {
    fn default() -> Self {
        Self::new()
    }
}

//! 运行时还原混淆公钥与 attestation 材料。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-16

include!(concat!(env!("OUT_DIR"), "/embedded_public_key.rs"));
include!(concat!(env!("OUT_DIR"), "/attestation_key.rs"));
include!(concat!(env!("OUT_DIR"), "/key_file_salt.rs"));

/// 嵌入材料访问器。
///
/// 功能：
///
/// - 还原编译期混淆的公钥 PEM
/// - 暴露 attestation HMAC 密钥与 `.key` 盐
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
pub struct EmbeddedMaterials;

impl EmbeddedMaterials {
    /// 构造访问器。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-16
    ///
    /// # 返回值
    ///
    /// 返回 [`EmbeddedMaterials`]。
    pub fn new() -> Self {
        Self
    }

    /// 还原公钥 PEM 字符串。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-16
    ///
    /// # 返回值
    ///
    /// 成功返回 PEM；字节非法 UTF-8 时返回错误。
    pub fn public_key_pem(&self) -> Result<String, String> {
        let mut bytes = Vec::new();
        let mut index: usize = 0;
        for chunk in OBFUSCATED_PUBLIC_KEY_CHUNKS {
            for byte in *chunk {
                let plain =
                    byte ^ PUBLIC_KEY_XOR_MASK ^ ((index as u8).wrapping_mul(PUBLIC_KEY_INDEX_MUL));
                bytes.push(plain);
                index += 1;
            }
        }
        String::from_utf8(bytes).map_err(|error| {
            format!(
                "deobfuscated public key is not UTF-8: {error}. Rebuild subscription after fixing keys/public_key.pem"
            )
        })
    }

    /// 返回 attestation HMAC 原始密钥。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-16
    ///
    /// # 返回值
    ///
    /// 32 字节密钥切片。
    pub fn attestation_key(&self) -> &'static [u8] {
        ATTESTATION_KEY
    }

    /// 返回 `.key` 文件 XOR 盐。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-16
    ///
    /// # 返回值
    ///
    /// 盐字节切片。
    pub fn key_file_salt(&self) -> &'static [u8] {
        KEY_FILE_OBFUSCATION_SALT
    }
}

impl Default for EmbeddedMaterials {
    fn default() -> Self {
        Self::new()
    }
}

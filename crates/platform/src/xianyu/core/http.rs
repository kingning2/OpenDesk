//! 闲鱼对外 HTTP 客户端 — `wreq` Chrome 仿真。
//!
//! `rquest` 在 crates.io 上已全部 yanked，同作者继任 crate 为 `wreq`，
//! API 兼容，用于 Chrome TLS / HTTP2 指纹伪装。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-19

use wreq::header::{HeaderMap, HeaderValue, REFERER, SET_COOKIE};
use wreq_util::Emulation;

use common::constants::xianyu;
use common::DingDaResult;

/// 构造闲鱼 HTTP 客户端（Chrome 133 TLS/HTTP2 指纹 + Cookie 存储）。
///
/// 不覆盖 User-Agent，避免与 `Emulation::Chrome133` 的指纹不一致。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-19
///
/// # 返回值
///
/// 可复用的 `wreq::Client`；构建失败返回错误文案。
pub fn build_client() -> DingDaResult<wreq::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(REFERER, HeaderValue::from_static(xianyu::WEB_ORIGIN));

    wreq::Client::builder()
        .emulation(Emulation::Chrome133)
        .default_headers(headers)
        .cookie_store(true)
        .build()
        .map_err(|error| format!("构建闲鱼 HTTP 客户端失败: {error}").into())
}

/// 从响应头收集 `Set-Cookie` 原文，供会话 Cookie 写回。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-19
///
/// # 参数
///
/// * `headers` - HTTP 响应头
///
/// # 返回值
///
/// 可解析的 Set-Cookie 字符串列表；非法 UTF-8 值会被跳过。
pub fn collect_set_cookies(headers: &HeaderMap) -> Vec<String> {
    headers
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .map(str::to_string)
        .collect()
}

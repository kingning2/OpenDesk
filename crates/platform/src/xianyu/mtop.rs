//! 闲鱼 mtop 通用 API 客户端。
//!
//! 收敛 Python 版 32 处重复的 mtop 调用模式：
//! 签名（md5 token&t&appKey&data）→ POST form → set-cookie 写回 → ret 校验 → TOKEN_EXPIRED 重试。
//! 业务层（评价/订单/发货/关单等）只声明 [`MtopRequest`]，客户端统一处理协议细节。
//!
//! 新接口发现与登记表：[`skills/dingda/guides/xianyu-mtop-discovery.md`](../../../../skills/dingda/guides/xianyu-mtop-discovery.md)
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-19

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use common::constants::xianyu;
use common::DingDaResult;

use super::cookie::{parse_cookies, sign_token};
use super::http::{build_client, collect_set_cookies};
use super::sign::generate_sign;

/// mtop 请求（业务层声明）。
#[derive(Debug, Clone)]
pub struct MtopRequest {
    /// 接口名，如 `mtop.taobao.idle.rate.create`。
    pub api: String,
    /// 接口版本，如 `4.0`。
    pub version: String,
    /// 业务数据（序列化为紧凑 JSON 作为 `data` 参数）。
    pub data: Value,
    /// 额外查询参数（如 spm、sessionOption 等）。
    pub extra_params: HashMap<String, String>,
}

impl MtopRequest {
    pub fn new(api: &str, version: &str, data: Value) -> Self {
        Self {
            api: api.to_string(),
            version: version.to_string(),
            data,
            extra_params: HashMap::new(),
        }
    }

    pub fn with_param(mut self, key: &str, value: &str) -> Self {
        self.extra_params.insert(key.to_string(), value.to_string());
        self
    }
}

/// mtop 响应。
#[derive(Debug, Clone)]
pub struct MtopResponse {
    /// 完整响应 JSON。
    pub json: Value,
    /// `ret[0]`（SUCCESS 或错误码）。
    pub ret: String,
}

impl MtopResponse {
    pub fn success(&self) -> bool {
        self.ret.contains("SUCCESS")
    }

    /// 取 `data` 节点。
    pub fn data(&self) -> Option<&Value> {
        self.json.get("data")
    }
}

/// mtop 客户端 — 持有可变 cookie 状态（set-cookie 写回 + 重试）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-19
#[derive(Clone)]
pub struct MtopClient {
    http: wreq::Client,
    /// 最新 cookie 字符串（set-cookie 写回后更新）。
    cookie: Arc<RwLock<String>>,
}

impl MtopClient {
    /// 用账号 Cookie 构造 mtop 客户端。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-19
    ///
    /// # 参数
    ///
    /// * `cookie_str` - 登录导出的 Cookie 原文
    ///
    /// # 返回值
    ///
    /// 成功返回客户端；HTTP 客户端构建失败返回错误。
    pub fn new(cookie_str: &str) -> DingDaResult<Self> {
        Ok(Self {
            http: build_client()?,
            cookie: Arc::new(RwLock::new(cookie_str.to_string())),
        })
    }

    /// 当前 cookie 字符串（业务层可读回并持久化）。
    pub async fn cookie(&self) -> String {
        self.cookie.read().await.clone()
    }

    /// 执行 mtop 请求；TOKEN_EXPIRED 时写回 set-cookie 并重试（最多 3 次）。
    pub async fn call(&self, request: &MtopRequest) -> DingDaResult<MtopResponse> {
        let mut retry = 0;
        loop {
            let response = self.call_once(request).await?;

            let token_expired =
                response.ret.contains("TOKEN_EXPIRED") || response.ret.contains("TOKEN_EXOIRED");
            if token_expired && retry < 2 {
                retry += 1;
                info!(
                    api = %request.api,
                    retry,
                    "mtop 令牌过期，已更新 Cookie，重试"
                );
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                continue;
            }
            return Ok(response);
        }
    }

    async fn call_once(&self, request: &MtopRequest) -> DingDaResult<MtopResponse> {
        let cookie_str = self.cookie.read().await.clone();
        let cookies = parse_cookies(&cookie_str);
        let token = sign_token(&cookies).unwrap_or_default();
        let timestamp = crate::xianyu::message::now_ms().to_string();
        let data_val = serde_json::to_string(&request.data)
            .map_err(|error| format!("mtop data 序列化失败: {error}"))?;
        let sign = generate_sign(&token, &timestamp, &data_val);

        // 公共查询参数。
        let mut params: HashMap<&str, String> = HashMap::new();
        params.insert("jsv", "2.7.2".to_string());
        params.insert("appKey", super::sign::APP_KEY.to_string());
        params.insert("t", timestamp.clone());
        params.insert("sign", sign);
        params.insert("v", request.version.clone());
        params.insert("type", "originaljson".to_string());
        params.insert("accountSite", "xianyu".to_string());
        params.insert("dataType", "json".to_string());
        params.insert("timeout", "20000".to_string());
        params.insert("api", request.api.clone());
        params.insert("sessionOption", "AutoLoginOnly".to_string());
        for (key, value) in &request.extra_params {
            params.insert(key, value.clone());
        }

        let url = format!(
            "{}{}/{}/",
            xianyu::H5_API_BASE,
            request.api,
            request.version
        );

        let response = self
            .http
            .post(&url)
            .query(&params)
            .header("Origin", xianyu::WEB_ORIGIN)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", cookie_str.clone())
            .form(&[("data", data_val.clone())])
            .send()
            .await
            .map_err(|error| format!("mtop 请求失败 ({}): {error}", request.api))?;

        // 写回 set-cookie（token 过期时服务端下发新 token）。
        let set_cookies = collect_set_cookies(response.headers());
        if !set_cookies.is_empty() {
            self.merge_set_cookies(&cookie_str, &set_cookies).await;
        }

        let json: Value = response
            .json()
            .await
            .map_err(|error| format!("mtop 响应解析失败: {error}"))?;

        let ret = json
            .get("ret")
            .and_then(Value::as_array)
            .and_then(|ret| ret.first())
            .and_then(Value::as_str)
            .map(|s| s.to_string())
            .unwrap_or_else(|| json.to_string());

        Ok(MtopResponse { json, ret })
    }

    /// 合并 set-cookie 到当前 cookie 字符串（按 name=value 覆盖）。
    async fn merge_set_cookies(&self, current: &str, set_cookies: &[String]) {
        let mut merged = parse_cookies(current);
        for cookie in set_cookies {
            if let Some((name, value)) = cookie.split_once('=') {
                let name = name.trim();
                if !name.is_empty() {
                    merged.insert(
                        name.to_string(),
                        value.split(';').next().unwrap_or_default().to_string(),
                    );
                }
            }
        }
        let updated: Vec<String> = merged
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect();
        let joined = updated.join("; ");
        if joined != current {
            info!(fields = merged.len(), "mtop 响应已更新 Cookie");
            *self.cookie.write().await = joined;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_builder() {
        let request = MtopRequest::new(
            "mtop.taobao.idle.rate.create",
            "4.0",
            serde_json::json!({
                "tradeId": "123",
                "rate": 1,
            }),
        )
        .with_param("spm_cnt", "a21ybx.0.0");
        assert_eq!(request.api, "mtop.taobao.idle.rate.create");
        assert_eq!(request.extra_params["spm_cnt"], "a21ybx.0.0");
        assert_eq!(request.data["tradeId"], "123");
    }

    #[test]
    fn response_helpers() {
        let ok = MtopResponse {
            json: serde_json::json!({ "ret": ["SUCCESS::调用成功"], "data": { "x": 1 } }),
            ret: "SUCCESS::调用成功".to_string(),
        };
        assert!(ok.success());
        assert_eq!(
            ok.data().and_then(|d| d.get("x")),
            Some(&serde_json::json!(1))
        );

        let fail = MtopResponse {
            json: serde_json::json!({ "ret": ["FAIL_SYS_TOKEN_EXPIRED"] }),
            ret: "FAIL_SYS_TOKEN_EXPIRED".to_string(),
        };
        assert!(!fail.success());
    }
}

//! 闲鱼 HTTP 接口 — Cookie 校验与 token 获取。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-19

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::http::{build_client, collect_set_cookies};
use super::sign::generate_sign;
use crate::shared::cookie::{device_id_from_cookie, my_id, now_ms, parse_cookies, sign_token};

use common::constants::xianyu;
use common::{DingDaError, DingDaResult};

const TOKEN_URL: &str = xianyu::LOGIN_TOKEN_URL;

/// 闲鱼 HTTP 客户端。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-19
#[derive(Clone)]
pub struct XianyuApi {
    http: wreq::Client,
    /// 可变 cookie 状态（响应 set-cookie 写回后更新，供签名重试使用）。
    cookie: Arc<RwLock<String>>,
}

impl XianyuApi {
    /// 用账号 Cookie 构造闲鱼 HTTP 客户端。
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
        let normalized = crate::shared::cookies::credential_to_cookie_header(cookie_str);
        Ok(Self {
            http: build_client()?,
            cookie: Arc::new(RwLock::new(normalized)),
        })
    }

    /// 当前 cookie 字符串（set-cookie 写回后为最新值）。
    pub async fn cookie_str(&self) -> String {
        self.cookie.read().await.clone()
    }

    /// 获取 WebSocket 注册 token。
    ///
    /// mtop 签名 token 取自 `_m_h5_tk` cookie 的 `_` 前缀。标准流程：
    /// 首次请求（无 token 或 token 过期）时服务端经 set-cookie 下发新 `_m_h5_tk`，
    /// 客户端需用新 token 重新签名后重试。本方法实现：
    /// 1. **预热**：cookie 缺 `_m_h5_tk` 时先发一次空签名请求，触发服务端下发；
    /// 2. **重试**：失败且 set-cookie 更新了 `_m_h5_tk`（token 变化）时，重签名再试（最多 3 次）。
    pub async fn fetch_token(&self) -> DingDaResult<String> {
        let mut current_token = self.sign_token_value().await;

        // 预热：cookie 缺 _m_h5_tk 时先请求一次，让服务端下发签名 token。
        if current_token.is_empty() {
            info!("cookie 缺少 _m_h5_tk，先发起探针请求获取签名 token");
            let _ = self.fetch_token_once().await;
            current_token = self.sign_token_value().await;
        }
        if current_token.is_empty() {
            return Err(DingDaError::validation(
                "cookie 缺少 _m_h5_tk，无法生成有效签名，请重新扫码登录",
            ));
        }

        let mut last_error = DingDaError::internal("token 获取失败");
        for attempt in 0..3 {
            match self.fetch_token_once().await {
                Ok(token) => return Ok(token),
                Err(error) => {
                    last_error = error;
                    // set-cookie 若下发了新 _m_h5_tk（token 变化），用新 token 重签名重试。
                    let refreshed = self.sign_token_value().await;
                    if refreshed.is_empty() || refreshed == current_token {
                        break;
                    }
                    info!(attempt = attempt + 1, "mtop token 已刷新，重新签名重试");
                    current_token = refreshed;
                }
            }
        }
        Err(last_error)
    }

    /// 当前 `_m_h5_tk` 的签名 token（`_` 前缀，缺失时为空串）。
    async fn sign_token_value(&self) -> String {
        let cookie_str = self.cookie.read().await.clone();
        sign_token(&parse_cookies(&cookie_str)).unwrap_or_default()
    }

    /// 执行一次 token 接口请求（带 set-cookie 写回）。
    async fn fetch_token_once(&self) -> DingDaResult<String> {
        let cookie_str = self.cookie.read().await.clone();
        let cookies = parse_cookies(&cookie_str);
        // 校验 cookie 完整性（unb 必须存在）。
        my_id(&cookies).ok_or_else(|| DingDaError::validation("cookie 缺少 unb"))?;
        let token = sign_token(&cookies).unwrap_or_default();
        let device_id = device_id_from_cookie(&cookie_str)
            .ok_or_else(|| DingDaError::validation("cookie 缺少 unb"))?;

        let data_val = format!(
            r#"{{"appKey":"{}","deviceId":"{}"}}"#,
            super::sign::REG_APP_KEY,
            device_id
        );
        let t = now_ms().to_string();
        let sign = generate_sign(&token, &t, &data_val);

        let mut params: HashMap<&str, String> = HashMap::new();
        params.insert("jsv", "2.7.2".to_string());
        params.insert("appKey", super::sign::APP_KEY.to_string());
        params.insert("t", t.clone());
        params.insert("sign", sign);
        params.insert("v", "1.0".to_string());
        params.insert("type", "originaljson".to_string());
        params.insert("accountSite", "xianyu".to_string());
        params.insert("dataType", "json".to_string());
        params.insert("timeout", "20000".to_string());
        params.insert("api", "mtop.taobao.idlemessage.pc.login.token".to_string());
        params.insert("sessionOption", "AutoLoginOnly".to_string());
        params.insert("spm_cnt", "a21ybx.im.0.0".to_string());
        params.insert("spm_pre", "a21ybx.item.want.1.14ad3da6ALVq3n".to_string());
        params.insert("log_id", "14ad3da6ALVq3n".to_string());

        let response = self
            .http
            .post(TOKEN_URL)
            .query(&params)
            .header("Origin", xianyu::WEB_ORIGIN)
            .header("Content-Type", "application/x-www-form-urlencoded")
            // 关键：携带账号 cookies，否则服务端识别不了会话，报 Session过期。
            .header("Cookie", cookie_str.clone())
            .form(&[("data", data_val.clone())])
            .send()
            .await
            .map_err(|error| format!("token 请求失败: {error}"))?;

        // 写回 set-cookie（token 过期/风控后服务端可能下发新 _m_h5_tk）。
        let set_cookies = collect_set_cookies(response.headers());
        if !set_cookies.is_empty() {
            self.merge_set_cookies(&cookie_str, &set_cookies).await;
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|error| format!("token 解析失败: {error}"))?;

        let ret_ok = body
            .get("ret")
            .and_then(serde_json::Value::as_array)
            .map(|ret| {
                ret.iter()
                    .any(|item| item.as_str().is_some_and(|s| s.contains("SUCCESS")))
            })
            .unwrap_or(false);

        if !ret_ok {
            return Err(format!("token 接口未成功: {body}").into());
        }

        Ok(body
            .pointer("/data/accessToken")
            .and_then(serde_json::Value::as_str)
            .map(|s| s.to_string())
            .ok_or_else(|| format!("token 接口缺 accessToken: {body}"))?)
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
            info!(fields = merged.len(), "token 响应已更新 Cookie");
            *self.cookie.write().await = joined;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sign_flow_fields() {
        let cookies = parse_cookies("unb=U-1; _m_h5_tk=tk_x; cna=cn; cookie2=c2");
        assert_eq!(my_id(&cookies), Some("U-1".to_string()));
        assert_eq!(sign_token(&cookies), Some("tk".to_string()));
        let sign = generate_sign("tk", "1000", "data");
        assert_eq!(sign.len(), 32);
    }
}

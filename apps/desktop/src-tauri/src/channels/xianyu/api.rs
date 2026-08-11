//! 闲鱼 HTTP 接口 — Cookie 校验与 token 获取。

use reqwest::header::{HeaderMap, HeaderValue, REFERER, USER_AGENT};
use std::collections::HashMap;

use super::cookie::{my_id, parse_cookies, sign_token};
use super::message::{device_id_from_cookie, now_ms};
use super::sign::generate_sign;

const TOKEN_URL: &str =
    "https://h5api.m.goofish.com/h5/mtop.taobao.idlemessage.pc.login.token/1.0/";
const HAS_LOGIN_URL: &str = "https://passport.goofish.com/newlogin/hasLogin.do";

/// 闲鱼 HTTP 客户端。
#[derive(Clone)]
pub struct XianyuApi {
    http: reqwest::Client,
    cookie_str: String,
}

impl XianyuApi {
    pub fn new(cookie_str: &str) -> Result<Self, String> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36"));
        headers.insert(REFERER, HeaderValue::from_static("https://www.goofish.com/"));

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .build()
            .map_err(|error| format!("构建闲鱼 HTTP 客户端失败: {error}"))?;

        Ok(Self {
            http,
            cookie_str: cookie_str.to_string(),
        })
    }

    /// 校验 Cookie 是否仍有效（`hasLogin.do` 返回 success）。
    #[allow(dead_code)]
    pub async fn has_login(&self) -> Result<bool, String> {
        let cookies = parse_cookies(&self.cookie_str);
        let hid = cookies.get("unb").cloned().unwrap_or_default();
        let csrf = cookies.get("XSRF-TOKEN").cloned().unwrap_or_default();
        let device = cookies.get("cna").cloned().unwrap_or_default();
        let cookie2 = cookies.get("cookie2").cloned().unwrap_or_default();

        let form: HashMap<String, String> = [
            ("appName", "xianyu"),
            ("fromSite", "77"),
            ("hid", hid.as_str()),
            ("ltl", "true"),
            ("appEntrance", "web"),
            ("_csrf_token", csrf.as_str()),
            ("umidToken", ""),
            ("hsiz", cookie2.as_str()),
            ("bizParams", "taobaoBizLoginFrom=web"),
            ("mainPage", "false"),
            ("isMobile", "false"),
            ("lang", "zh_CN"),
            ("returnUrl", ""),
            ("isIframe", "true"),
            ("documentReferer", "https://www.goofish.com/"),
            ("defaultView", "hasLogin"),
            ("umidTag", "SERVER"),
            ("deviceId", device.as_str()),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

        let response = self
            .http
            .post(HAS_LOGIN_URL)
            .query(&[("appName", "xianyu"), ("fromSite", "77")])
            .header("Origin", "https://www.goofish.com")
            .form(&form)
            .send()
            .await
            .map_err(|error| format!("hasLogin 请求失败: {error}"))?;

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|error| format!("hasLogin 解析失败: {error}"))?;

        Ok(body
            .pointer("/content/success")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false))
    }

    /// 获取 WebSocket 注册 token。
    pub async fn fetch_token(&self) -> Result<String, String> {
        let cookies = parse_cookies(&self.cookie_str);
        // 校验 cookie 完整性（unb 必须存在）。
        my_id(&cookies).ok_or_else(|| "cookie 缺少 unb".to_string())?;
        let token = sign_token(&cookies).unwrap_or_default();
        let device_id = device_id_from_cookie(&self.cookie_str)
            .ok_or_else(|| "cookie 缺少 unb".to_string())?;

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
            .header("Origin", "https://www.goofish.com")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[("data", data_val.clone())])
            .send()
            .await
            .map_err(|error| format!("token 请求失败: {error}"))?;

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|error| format!("token 解析失败: {error}"))?;

        let ret_ok = body
            .get("ret")
            .and_then(serde_json::Value::as_array)
            .map(|ret| ret.iter().any(|item| item.as_str().is_some_and(|s| s.contains("SUCCESS"))))
            .unwrap_or(false);

        if !ret_ok {
            return Err(format!("token 接口未成功: {body}"));
        }

        body.pointer("/data/accessToken")
            .and_then(serde_json::Value::as_str)
            .map(|s| s.to_string())
            .ok_or_else(|| format!("token 接口缺 accessToken: {body}"))
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

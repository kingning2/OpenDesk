//! 会话关联商品卡信息 — `mtop.idle.trade.pc.message.headinfo`（GET）。
//!
//! 闲鱼网页端打开聊天时用 GET 请求该接口，`data={"itemId":..., "sessionId":..., "sessionType":1}`，
//! 返回会话关联的商品信息（标题/图/价格等）。字段以实际返回为准，未固化。

use common::DingDaResult;
use serde_json::Value;

use super::mtop::{MtopClient, MtopRequest};

/// 拉取会话关联商品卡信息，返回响应 `data` 节点（原始 JSON）。
///
/// # 参数
/// - `cookie_str` — 账号 Cookie 原文
/// - `session_id` — 会话对端用户 id（会话同步里的 peer_id）
/// - `item_id` — 会话关联商品 id
pub async fn fetch_message_headinfo(
    cookie_str: &str,
    session_id: &str,
    item_id: &str,
) -> DingDaResult<Value> {
    let mut data = serde_json::json!({
        "sessionId": session_id,
        "sessionType": 1,
    });
    if let Ok(id) = item_id.parse::<i64>() {
        data["itemId"] = serde_json::json!(id);
    }
    let client = MtopClient::new(cookie_str)?;
    let request = MtopRequest::new("mtop.idle.trade.pc.message.headinfo", "1.0", data)
        .with_get()
        .with_param("spm_cnt", "a21ybx.im.0.0");
    let response = client.call(&request).await?;
    if !response.success() {
        return Err(format!("message.headinfo 未成功: {}", response.ret).into());
    }
    Ok(response.data().cloned().unwrap_or(Value::Null))
}

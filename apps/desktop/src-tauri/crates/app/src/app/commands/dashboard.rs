//! 首页数据看板 Tauri IPC command（跨库聚合统计）。
//!
//! 作者：coisini
//! 创建时间：2026-08-08

use common::contracts::{DashboardIpcStatsRequest, DashboardIpcStatsResponse};
use ports::crawler_channels::ChannelStats;
use ports::customer::CustomerListQuery;
use ports::mail::{MailMessageListFilter, MailStore};
use serde_json::{json, Value};

use crate::app::state::AppState;

const MAIL_DIRECTION_INBOUND: &str = "inbound";
const MAIL_DIRECTION_OUTBOUND: &str = "outbound";

/// Aggregate crawler/customer/mail stats for the home dashboard.
///
/// 作者：coisini
/// 创建时间：2026-08-08
#[tauri::command]
pub async fn dashboard_stats(
    state: tauri::State<'_, AppState>,
    request: DashboardIpcStatsRequest,
) -> Result<DashboardIpcStatsResponse, String> {
    let channels_store = state.channels_store.clone();
    let customer_store = state.customer_store.clone();
    let mail_store = state.mail_store.clone();
    let trace_id = request.trace_id.clone();

    let stats = tauri::async_runtime::spawn_blocking(move || -> Result<Value, String> {
        let channel = channels_store.stats().map_err(|error| error.to_string())?;
        let customer_total = customer_store
            .list(CustomerListQuery {
                search: None,
                limit: 1,
                offset: 0,
            })
            .map_err(|error| error.to_string())?
            .total;
        let mail_total = count_mail_messages(mail_store.as_ref())?;
        Ok(dashboard_json(channel, customer_total, mail_total))
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())?;

    Ok(DashboardIpcStatsResponse {
        ok: true,
        stats_json: serde_json::to_string(&stats).map_err(|error| error.to_string())?,
        trace_id,
    })
}

/// Sum `mail_message` rows across inbound + outbound directions.
fn count_mail_messages(mail_store: &dyn MailStore) -> Result<i64, String> {
    let mut total = 0i64;
    for direction in [MAIL_DIRECTION_INBOUND, MAIL_DIRECTION_OUTBOUND] {
        let (_, count) = mail_store
            .list_messages(MailMessageListFilter {
                direction: direction.to_string(),
                account_id: None,
                customer_id: None,
                query: None,
                limit: 1,
                offset: 0,
            })
            .map_err(|error| error.to_string())?;
        total += count;
    }
    Ok(total)
}

/// Build the dashboard payload object (serialized into `stats_json`).
fn dashboard_json(channel: ChannelStats, customer_total: i64, mail_total: i64) -> Value {
    let by_platform = channel
        .by_platform
        .iter()
        .map(|bucket| json!({ "key": bucket.key, "count": bucket.count }))
        .collect::<Vec<_>>();
    let by_email_status = channel
        .by_email_status
        .iter()
        .map(|bucket| json!({ "key": bucket.key, "count": bucket.count }))
        .collect::<Vec<_>>();

    json!({
        "total_channels": channel.total_channels,
        "total_emails": channel.total_emails,
        "total_verified_emails": channel.total_verified_emails,
        "by_platform": by_platform,
        "by_email_status": by_email_status,
        "customer_total": customer_total,
        "mail_total": mail_total,
    })
}

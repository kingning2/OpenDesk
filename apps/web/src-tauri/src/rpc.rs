//! RPC 分发表：把 `POST /api/rpc` 的 `{command, args}` 分发到领域 use-case。
//!
//! 每个 handler 与桌面 `crates/app` 中对应 Tauri command 同源：clone store +
//! `tokio::task::spawn_blocking` + 调领域 crate 的 use-case，返回与桌面完全一致的
//! 契约响应结构，前端 `invokeIpc` 的 fetch shim 可原样消费。

use app_core::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::llm::stored_llm_client;
use crate::sse::SseChatEmitter;
use crate::ServerState;

/// RPC 请求体。
#[derive(serde::Deserialize)]
pub struct RpcRequest {
    pub command: String,
    #[serde(default)]
    pub args: Value,
}

/// RPC 响应。
fn ok_response(data: Value) -> Value {
    json!({ "ok": true, "data": data })
}

fn err_response(message: impl Into<String>) -> Value {
    json!({ "ok": false, "error": message.into() })
}

/// 从 `args.request` 反序列化契约请求类型；缺字段时容忍。
fn request_of<T: DeserializeOwned>(args: &Value) -> Result<T, String> {
    let request = args.get("request").cloned().unwrap_or(Value::Null);
    serde_json::from_value(request).map_err(|error| format!("invalid request: {error}"))
}

/// RPC 入口：按 command 名分发。
pub async fn rpc(State(state): State<ServerState>, Json(body): Json<RpcRequest>) -> Response {
    let result = dispatch(&state.app, &state.chat_emitter, &body.command, &body.args).await;
    match result {
        Ok(value) => (StatusCode::OK, Json(ok_response(value))).into_response(),
        Err(message) => (StatusCode::BAD_REQUEST, Json(err_response(message))).into_response(),
    }
}

/// 未实现命令的错误。
fn not_implemented(command: &str) -> Result<Value, String> {
    Err(format!("command '{command}' not yet implemented in web"))
}

async fn dispatch(
    app: &AppState,
    chat_emitter: &SseChatEmitter,
    command: &str,
    args: &Value,
) -> Result<Value, String> {
    match command {
        // 看板
        "dashboard_stats" => dashboard_stats(app).await,

        // 客户
        "customer_list" => {
            let req = request_of::<common::contracts::CustomerIpcListRequest>(args)?;
            let store = app.customer_store.clone();
            blocking(move || customer::app::ListCustomers::execute(store.as_ref(), req)).await
        }
        "customer_get" => {
            let req = request_of::<common::contracts::CustomerIpcGetRequest>(args)?;
            let store = app.customer_store.clone();
            blocking(move || customer::app::GetCustomer::execute(store.as_ref(), req)).await
        }
        "customer_create" => {
            let req = request_of::<common::contracts::CustomerIpcCreateRequest>(args)?;
            let store = app.customer_store.clone();
            blocking(move || customer::app::CreateCustomer::execute(store.as_ref(), req)).await
        }
        "customer_update" => {
            let req = request_of::<common::contracts::CustomerIpcUpdateRequest>(args)?;
            let store = app.customer_store.clone();
            blocking(move || customer::app::UpdateCustomer::execute(store.as_ref(), req)).await
        }

        // 爬虫 channel
        "crawler_channel_list" => {
            let req = request_of::<common::contracts::CrawlerIpcChannelListRequest>(args)?;
            let store = app.channels_store.clone();
            blocking(move || crate::crawler::channel_list(store.as_ref(), req)).await
        }
        "crawler_channel_update" => {
            let req = request_of::<common::contracts::CrawlerIpcChannelUpdateRequest>(args)?;
            let store = app.channels_store.clone();
            blocking(move || crate::crawler::channel_update(store.as_ref(), req)).await
        }

        // 爬虫 job
        "crawler_job_start" => {
            let req = request_of::<common::contracts::CrawlerIpcJobStartRequest>(args)?;
            crate::crawler::job_start(app, req).await
        }
        "crawler_job_cancel" => {
            let req = request_of::<common::contracts::CrawlerIpcJobCancelRequest>(args)?;
            Ok(json!(app.crawler.cancel(req).map_err(|e| e.to_string())?))
        }
        "crawler_job_status" => {
            let req = request_of::<common::contracts::CrawlerIpcJobStatusRequest>(args)?;
            Ok(json!(app.crawler.status(req).map_err(|e| e.to_string())?))
        }
        "crawler_job_logs" => {
            let req = request_of::<common::contracts::CrawlerIpcJobLogsRequest>(args)?;
            Ok(json!(app.crawler.logs(req).map_err(|e| e.to_string())?))
        }
        "crawler_job_results" => {
            let req = request_of::<common::contracts::CrawlerIpcJobResultsRequest>(args)?;
            crate::crawler::job_results(app, req).await
        }

        // 爬虫 keywords
        "crawler_keywords_import" => {
            let req = request_of::<common::contracts::CrawlerIpcKeywordsImportRequest>(args)?;
            let store = app.keywords_store.clone();
            blocking(move || {
                let result = store
                    .import_csv(&req.csv_content, req.batch_id.as_deref())
                    .map_err(|error| error.to_string())?;
                Ok::<Value, String>(json!({
                    "ok": true,
                    "batch_id": result.batch_id,
                    "inserted": result.inserted,
                    "skipped_existing": result.skipped_existing,
                    "skipped_too_long": result.skipped_too_long,
                    "total": result.total,
                    "trace_id": req.trace_id,
                    "message": null,
                }))
            })
            .await
        }
        "crawler_keywords_batches" => {
            let store = app.keywords_store.clone();
            blocking(move || {
                let batches = store.list_batches().map_err(|error| error.to_string())?;
                let payload: Vec<Value> = batches
                    .into_iter()
                    .map(|item| json!({ "batch_id": item.batch_id, "keyword_count": item.keyword_count }))
                    .collect();
                Ok::<_, String>(json!({
                    "ok": true,
                    "batches_json": serde_json::to_string(&payload).map_err(|e| e.to_string())?,
                    "trace_id": null,
                }))
            })
            .await
        }
        "crawler_keywords_generate" => not_implemented(command),
        "crawler_youtube_api_key_get" => {
            let store = app.settings_store.clone();
            blocking(move || crate::crawler::youtube_api_key_get(store.as_ref())).await
        }
        "crawler_youtube_api_key_set" => {
            let req = args.get("request").cloned().unwrap_or(Value::Null);
            let store = app.settings_store.clone();
            blocking(move || crate::crawler::youtube_api_key_set(store.as_ref(), req)).await
        }

        // LLM
        "llm_settings_get" => crate::llm::settings_get(app).await,
        "llm_settings_save" => {
            let req = request_of::<common::contracts::RuntimeIpcLlmSettingsSaveRequest>(args)?;
            crate::llm::settings_save(app, req).await
        }
        "llm_test_connection" => {
            let req = request_of::<common::contracts::RuntimeIpcLlmTestConnectionRequest>(args)?;
            crate::llm::test_connection(app, req).await
        }

        // License
        "license_status" => {
            let status = app.license.status().await.map_err(|e| e.to_string())?;
            Ok(json!(status))
        }
        "license_machine_code" => {
            let code = app
                .license
                .machine_code()
                .await
                .map_err(|e| e.to_string())?;
            Ok(json!(code))
        }
        "license_activate" => {
            let req = args.get("request").cloned().unwrap_or(Value::Null);
            let req: common::license::LicenseActivateRequest =
                serde_json::from_value(req).map_err(|e| e.to_string())?;
            let result = app.license.activate(req).await.map_err(|e| e.to_string())?;
            Ok(json!(result))
        }

        // Mail
        "mail_template_list" => {
            let store = app.mail_store.clone();
            blocking(move || mail::app::ListMailTemplates::execute(store.as_ref())).await
        }
        "mail_template_save" => {
            let req = request_of::<common::contracts::MailIpcTemplateSaveRequest>(args)?;
            let store = app.mail_store.clone();
            blocking(move || mail::app::SaveMailTemplate::execute(store.as_ref(), req)).await
        }
        "mail_template_apply" => {
            let req = request_of::<common::contracts::MailIpcTemplateApplyRequest>(args)?;
            let mail_store = app.mail_store.clone();
            let customer_store = app.customer_store.clone();
            blocking(move || {
                mail::app::ApplyMailTemplate::execute(
                    mail_store.as_ref(),
                    customer_store.as_ref(),
                    req,
                )
            })
            .await
        }
        "mail_account_list" => {
            let store = app.mail_store.clone();
            blocking(move || mail::app::ListMailAccounts::execute(store.as_ref())).await
        }
        "mail_account_save" => {
            let req = request_of::<common::contracts::MailIpcAccountSaveRequest>(args)?;
            let store = app.mail_store.clone();
            blocking(move || mail::app::SaveMailAccount::execute(store.as_ref(), req)).await
        }
        "mail_account_delete" => {
            let req = request_of::<common::contracts::MailIpcAccountDeleteRequest>(args)?;
            let store = app.mail_store.clone();
            blocking(move || mail::app::DeleteMailAccount::execute(store.as_ref(), req)).await
        }
        "mail_message_list" => {
            let req = request_of::<common::contracts::MailIpcMessageListRequest>(args)?;
            let store = app.mail_store.clone();
            blocking(move || mail::app::ListMailMessages::execute(store.as_ref(), req)).await
        }
        "mail_send" => {
            let req = request_of::<common::contracts::MailIpcSendRequest>(args)?;
            let mail_store = app.mail_store.clone();
            let customer_store = app.customer_store.clone();
            blocking(move || {
                mail::app::SendMail::execute(mail_store.as_ref(), customer_store.as_ref(), req)
            })
            .await
        }
        "mail_record_inbound" => {
            let req = request_of::<common::contracts::MailIpcRecordInboundRequest>(args)?;
            let mail_store = app.mail_store.clone();
            let customer_store = app.customer_store.clone();
            blocking(move || {
                mail::app::RecordInboundMail::execute(
                    mail_store.as_ref(),
                    customer_store.as_ref(),
                    req,
                )
            })
            .await
        }
        "mail_sync_now" => {
            let req = request_of::<common::contracts::MailIpcSyncNowRequest>(args)?;
            let job_store = app.job_store.clone();
            let mail_store = app.mail_store.clone();
            let customer_store = app.customer_store.clone();
            blocking(move || {
                mail::app::SyncMailNow::execute(
                    job_store.as_ref(),
                    mail_store.as_ref(),
                    customer_store.as_ref(),
                    req,
                )
            })
            .await
        }
        "mail_sync_status" => {
            let req = request_of::<common::contracts::MailIpcSyncStatusRequest>(args)?;
            let job_store = app.job_store.clone();
            let mail_store = app.mail_store.clone();
            blocking(move || {
                mail::app::GetMailSyncStatus::execute(job_store.as_ref(), mail_store.as_ref(), req)
            })
            .await
        }
        "mail_inbox_unmatched_list" => {
            let req = request_of::<common::contracts::MailIpcInboxUnmatchedListRequest>(args)?;
            let store = app.mail_store.clone();
            blocking(move || mail::app::ListUnmatchedInbound::execute(store.as_ref(), req)).await
        }
        "mail_link_inbound_customer" => {
            let req = request_of::<common::contracts::MailIpcLinkInboundCustomerRequest>(args)?;
            let mail_store = app.mail_store.clone();
            let customer_store = app.customer_store.clone();
            blocking(move || {
                mail::app::LinkInboundCustomer::execute(
                    mail_store.as_ref(),
                    customer_store.as_ref(),
                    req,
                )
            })
            .await
        }
        "mail_generate_html" => {
            let req = request_of::<common::contracts::MailIpcGenerateHtmlRequest>(args)?;
            let client = stored_llm_client(app).await?;
            let generated = mail::app::GenerateMailHtml::execute(&client, &req.text).await?;
            Ok(json!(common::contracts::MailIpcGenerateHtmlResponse {
                ok: true,
                html: generated.html,
                notes: generated.notes,
                message: "ok".to_string(),
                trace_id: req.trace_id,
            }))
        }

        // Chat 会话管理
        "chat_send" => chat_send(app, chat_emitter, args).await,
        "chat_session_list" => {
            let store = app.chat_store.clone();
            blocking(move || {
                let sessions = store
                    .list_sessions()
                    .map_err(|error| error.to_string())?
                    .into_iter()
                    .map(session_to_dto)
                    .collect::<Vec<_>>();
                let sessions_json = serde_json::to_string(&sessions).map_err(|e| e.to_string())?;
                Ok::<_, String>(json!({ "sessions_json": sessions_json }))
            })
            .await
        }
        "chat_session_create" => {
            let req = request_of::<common::contracts::ChatIpcSessionCreateRequest>(args)?;
            let store = app.chat_store.clone();
            blocking(move || {
                let id = uuid::Uuid::new_v4().to_string();
                let session = store
                    .create_session(&id, req.title.as_deref().unwrap_or(""))
                    .map_err(|error| error.to_string())?;
                let session_json =
                    serde_json::to_string(&session_to_dto(session)).map_err(|e| e.to_string())?;
                Ok::<_, String>(json!({ "session_json": session_json }))
            })
            .await
        }
        "chat_session_rename" => {
            let req = request_of::<common::contracts::ChatIpcSessionRenameRequest>(args)?;
            let store = app.chat_store.clone();
            blocking(move || {
                let session = store
                    .rename_session(&req.id, &req.title)
                    .map_err(|error| error.to_string())?;
                let session_json =
                    serde_json::to_string(&session_to_dto(session)).map_err(|e| e.to_string())?;
                Ok::<_, String>(json!({ "session_json": session_json }))
            })
            .await
        }
        "chat_session_delete" => {
            let req = request_of::<common::contracts::ChatIpcSessionDeleteRequest>(args)?;
            let store = app.chat_store.clone();
            blocking(move || {
                store
                    .delete_session(&req.id)
                    .map_err(|error| error.to_string())?;
                Ok::<_, String>(json!({ "ok": true }))
            })
            .await
        }
        "chat_messages_load" => {
            let req = request_of::<common::contracts::ChatIpcMessagesLoadRequest>(args)?;
            let store = app.chat_store.clone();
            blocking(move || {
                let messages = store
                    .load_messages(&req.session_id)
                    .map_err(|error| error.to_string())?
                    .into_iter()
                    .map(message_to_dto)
                    .collect::<Vec<_>>();
                let messages_json = serde_json::to_string(&messages).map_err(|e| e.to_string())?;
                Ok::<_, String>(json!({ "messages_json": messages_json }))
            })
            .await
        }

        // 知识库
        "knowledge_doc_import" => {
            let req = request_of::<common::contracts::KnowledgeIpcDocumentImportRequest>(args)?;
            crate::knowledge::doc_import(app, req).await
        }
        "knowledge_doc_list" => crate::knowledge::doc_list(app).await,
        "knowledge_doc_delete" => {
            let req = request_of::<common::contracts::KnowledgeIpcDocumentDeleteRequest>(args)?;
            crate::knowledge::doc_delete(app, req).await
        }
        "knowledge_tool_status" => crate::knowledge::tool_status().await,
        "knowledge_tool_download" => not_implemented(command),

        // 其它未实现
        "help_ask"
        | "agent_ping"
        | "mail_email_read_integration_get"
        | "mail_email_read_integration_save"
        | "mail_email_read_integration_probe"
        | "workflow_template_list"
        | "workflow_template_get"
        | "workflow_binding_list"
        | "workflow_rule_list"
        | "workflow_script_list"
        | "workflow_runtime_start"
        | "workflow_runtime_cancel"
        | "workflow_runtime_resume"
        | "workflow_runtime_active" => not_implemented(command),

        _ => not_implemented(command),
    }
}

/// 在 blocking 线程池执行同步 use-case，返回其 `serde::Serialize` 响应。
async fn blocking<T, F>(f: F) -> Result<Value, String>
where
    T: serde::Serialize + Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    let value = tokio::task::spawn_blocking(f)
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;
    serde_json::to_value(value).map_err(|error| error.to_string())
}

/// dashboard_stats 聚合（与桌面 command 同逻辑）。
async fn dashboard_stats(app: &AppState) -> Result<Value, String> {
    let channels_store = app.channels_store.clone();
    let customer_store = app.customer_store.clone();
    let mail_store = app.mail_store.clone();
    let stats = blocking(move || {
        let channel = channels_store.stats().map_err(|error| error.to_string())?;
        let customer_total = customer_store
            .list(ports::customer::CustomerListQuery {
                search: None,
                limit: 1,
                offset: 0,
            })
            .map_err(|error| error.to_string())?
            .total;
        let mail_total = count_mail_messages(mail_store.as_ref())?;
        Ok::<_, String>(dashboard_json(channel, customer_total, mail_total))
    })
    .await?;
    let stats_json = serde_json::to_string(&stats).map_err(|error| error.to_string())?;
    Ok(json!({ "ok": true, "stats_json": stats_json }))
}

/// 跨 inbound/outbound 汇总邮件消息数。
fn count_mail_messages(mail_store: &dyn ports::mail::MailStore) -> Result<i64, String> {
    let mut total = 0i64;
    for direction in ["inbound", "outbound"] {
        let (_, count) = mail_store
            .list_messages(ports::mail::MailMessageListFilter {
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

/// 构造 dashboard 载荷对象（与桌面 `dashboard_json` 同构）。
fn dashboard_json(
    channel: ports::crawler_channels::ChannelStats,
    customer_total: i64,
    mail_total: i64,
) -> Value {
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

fn session_to_dto(record: ports::chat::ChatSessionRecord) -> common::contracts::ChatDtoSession {
    common::contracts::ChatDtoSession {
        id: record.id,
        title: record.title,
        created_at: record.created_at,
        updated_at: record.updated_at,
        last_message_at: record.last_message_at,
        message_count: record.message_count,
    }
}

fn message_to_dto(record: ports::chat::ChatMessageRecord) -> common::contracts::ChatDtoMessage {
    common::contracts::ChatDtoMessage {
        id: record.id,
        session_id: record.session_id,
        role: record.role,
        content: record.content,
        thinking: record.thinking,
        tools_json: record.tools_json,
        seq: record.seq,
        created_at: record.created_at,
    }
}

/// 从 LLM 设置读取开关（未配置默认开启）。
async fn settings_flag(
    app: &AppState,
    field: fn(&ports::llm_settings::LlmSettingsRecord) -> bool,
) -> Result<bool, String> {
    let store = app.llm_settings_store.clone();
    tokio::task::spawn_blocking(move || {
        let enabled = store
            .get()
            .map_err(|error| error.to_string())?
            .map(|record| field(&record))
            .unwrap_or(true);
        Ok::<bool, String>(enabled)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())
}

/// chat_send：回复 token 经 SSE `chat:message/token` / `chat:message/tool` 推送。
async fn chat_send(
    app: &AppState,
    emitter: &SseChatEmitter,
    args: &Value,
) -> Result<Value, String> {
    let req = request_of::<common::contracts::ChatIpcSendRequest>(args)?;
    tracing::info!(
        session_id = %req.session_id,
        message_id = %req.message_id.as_deref().unwrap_or("-"),
        "chat_send: request received"
    );
    let client = stored_llm_client(app).await?;

    let allow_memory = settings_flag(app, |r| r.memory_enabled).await?;
    let memory = if allow_memory {
        Some(app.chat_memory_store.clone())
    } else {
        None
    };
    let embedder = if allow_memory {
        Some(app.embedder.clone())
    } else {
        None
    };
    let allow_knowledge = settings_flag(app, |r| r.knowledge_enabled).await?;
    let knowledge = if allow_knowledge {
        Some(app.knowledge_store.clone())
    } else {
        None
    };
    let session_id = req.session_id.clone();
    let response = chat::SendChat::execute(
        &client,
        emitter,
        req,
        None,
        None,
        Some(app.chat_store.as_ref()),
        memory,
        embedder,
        knowledge,
    )
    .await?;

    if allow_memory {
        let digest_client = client.clone();
        let digest_store = app.chat_store.clone();
        let digest_memory = app.chat_memory_store.clone();
        let digest_embedder = app.embedder.clone();
        tokio::spawn(async move {
            if let Err(error) = chat::maybe_digest(
                &digest_client,
                digest_store.as_ref(),
                digest_memory.as_ref(),
                digest_embedder,
                session_id,
            )
            .await
            {
                tracing::warn!(%error, "chat memory digest failed");
            }
        });
    }

    Ok(json!(response))
}

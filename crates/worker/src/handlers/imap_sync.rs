//! Handle `imap_sync` background jobs.
//!
//! 作者：coisini
//! 创建时间：2026-07-22

use std::sync::Arc;

use mail::app::RunImapAccountSync;
use ports::background_job::{BackgroundJobRecord, ImapSyncPayload};
use ports::customer::CustomerStore;
use ports::mail::MailStore;
use thiserror::Error;

const DEFAULT_FOLDER: &str = "INBOX";

#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("invalid payload: {0}")]
    InvalidPayload(String),
    #[error("store error: {0}")]
    Store(#[from] ports::repository::StoreError),
    #[error("imap error: {0}")]
    Imap(String),
}

/// Execute one IMAP inbox sync job.
///
/// 作者：coisini
/// 创建时间：2026-07-22
pub async fn handle(
    job: &BackgroundJobRecord,
    mail_store: Arc<dyn MailStore>,
    customer_store: Arc<dyn CustomerStore>,
) -> Result<(), HandlerError> {
    let payload: ImapSyncPayload = serde_json::from_str(&job.payload_json)
        .map_err(|error| HandlerError::InvalidPayload(error.to_string()))?;

    let folder = if payload.folder.trim().is_empty() {
        DEFAULT_FOLDER.to_string()
    } else {
        payload.folder.clone()
    };

    RunImapAccountSync::execute(
        mail_store.as_ref(),
        customer_store.as_ref(),
        &payload.account_id,
        &folder,
    )
    .map_err(HandlerError::Imap)
}

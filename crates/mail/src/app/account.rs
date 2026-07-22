//! Mail account list and save use cases.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-21

use common::contracts::{MailIpcAccountListResponse, MailIpcAccountSaveRequest};
use ports::mail::{MailAccountWriteInput, MailStore};

use super::mapper::accounts_to_json;

/// List saved mail accounts.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
pub struct ListMailAccounts;

impl ListMailAccounts {
    /// Load all configured mail accounts.
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-21
    pub fn execute<S: MailStore + ?Sized>(store: &S) -> Result<MailIpcAccountListResponse, String> {
        let accounts = store.list_accounts().map_err(|error| error.to_string())?;
        Ok(MailIpcAccountListResponse {
            accounts_json: accounts_to_json(&accounts)?,
            total: accounts.len() as i64,
        })
    }
}

/// Save one mail account.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
pub struct SaveMailAccount;

impl SaveMailAccount {
    /// Create or update one mail account configuration.
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-21
    pub fn execute<S: MailStore + ?Sized>(
        store: &S,
        request: MailIpcAccountSaveRequest,
    ) -> Result<MailIpcAccountListResponse, String> {
        if request.password.trim().is_empty() {
            return Err("mail.account.password_required".to_string());
        }

        store
            .save_account(MailAccountWriteInput {
                id: request.id,
                label: request.label,
                from_address: request.from_address,
                from_name: request.from_name,
                smtp_host: request.smtp_host,
                smtp_port: request.smtp_port,
                use_tls: request.use_tls,
                username: request.username,
                password: request.password,
                imap_host: request.imap_host,
                imap_port: request.imap_port,
                imap_use_tls: request.imap_use_tls,
                imap_sync_enabled: request.imap_sync_enabled,
            })
            .map_err(|error| error.to_string())?;

        ListMailAccounts::execute(store)
    }
}

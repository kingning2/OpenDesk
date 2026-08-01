//! Mail application use cases.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-21

mod account;
mod generate_html;
mod imap_idle;
mod imap_inbound;
mod imap_sync;
mod inbound;
mod integration_settings;
mod list;
mod mapper;
mod mark_read;
mod send;
mod template;
mod tracking;

pub use account::{ListMailAccounts, SaveMailAccount};
pub use generate_html::{
    parse_email_html_response, GenerateMailHtml, GeneratedMailHtml, EMAIL_HTML_SYSTEM_PROMPT,
};
pub use imap_idle::{spawn_imap_idle_watchers, watch_account_idle, ImapIdlePersistHook};
pub use imap_inbound::{
    next_imap_sync_cursor, persist_imap_fetched_messages, should_persist_inbound,
    PersistImapInboundSummary, IMAP_SYNC_MAX_FETCH,
};
pub use imap_sync::{
    GetMailSyncStatus, LinkInboundCustomer, ListUnmatchedInbound, RunImapAccountSync,
    ScheduleImapSync, SyncMailNow,
};
pub use inbound::RecordInboundMail;
pub use integration_settings::{
    GetEmailReadIntegration, ProbeEmailReadIntegration, SaveEmailReadIntegration,
};
pub use list::ListMailMessages;
pub use mark_read::MarkMailMessageRead;
pub use send::SendMail;
pub use template::{ApplyMailTemplate, ListMailTemplates, SaveMailTemplate};

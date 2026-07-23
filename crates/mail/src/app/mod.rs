//! Mail application use cases.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-21

mod account;
mod imap_sync;
mod inbound;
mod list;
mod mapper;
mod send;
mod template;
mod tracking;

pub use account::{ListMailAccounts, SaveMailAccount};
pub use imap_sync::{
    GetMailSyncStatus, LinkInboundCustomer, ListUnmatchedInbound, ScheduleImapSync, SyncMailNow,
};
pub use inbound::RecordInboundMail;
pub use list::ListMailMessages;
pub use send::SendMail;
pub use template::{ApplyMailTemplate, ListMailTemplates, SaveMailTemplate};

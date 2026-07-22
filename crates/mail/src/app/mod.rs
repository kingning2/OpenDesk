//! Mail application use cases.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-21

mod account;
mod inbound;
mod mapper;
mod send;
mod template;

pub use account::{ListMailAccounts, SaveMailAccount};
pub use inbound::RecordInboundMail;
pub use send::SendMail;
pub use template::{ApplyMailTemplate, ListMailTemplates};

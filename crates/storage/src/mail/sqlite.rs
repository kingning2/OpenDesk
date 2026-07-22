//! Diesel-backed mail store.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-21

use diesel::prelude::*;
use keyring::Entry;
use ports::mail::{
    mail_keyring_user, mail_password_ref, MailAccountRecord, MailAccountWriteInput,
    MailInboundWriteInput, MailMessageListFilter, MailMessageRecord, MailSendInput, MailStore,
    MailTemplateRecord, MailTemplateWriteInput, MAIL_KEYRING_SERVICE,
};
use ports::repository::StoreError;
use uuid::Uuid;

use crate::opendesk_db::schema::customer_timeline::dsl as timeline;
use crate::opendesk_db::schema::mail_account::dsl as account;
use crate::opendesk_db::schema::mail_message::dsl as message;
use crate::opendesk_db::schema::mail_template::dsl as template;
use crate::opendesk_db::{
    MailAccountRow, MailMessageRow, MailTemplateRow, NewMailAccountRow, NewMailMessageRow,
    NewMailTemplateRow, OpendeskDb,
};

/// SQLite implementation of [`MailStore`].
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
pub struct SqliteMailStore {
    db: OpendeskDb,
}

impl SqliteMailStore {
    /// Wrap an existing [`OpendeskDb`] handle.
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-21
    pub fn new(db: OpendeskDb) -> Self {
        Self { db }
    }

    fn keyring_entry(account_id: &str) -> Result<Entry, StoreError> {
        Entry::new(MAIL_KEYRING_SERVICE, &mail_keyring_user(account_id))
            .map_err(|error| StoreError::Unavailable(format!("keyring entry: {error}")))
    }

    fn keyring_set(account_id: &str, secret: &str) -> Result<(), StoreError> {
        let entry = Self::keyring_entry(account_id)?;
        entry
            .set_password(secret)
            .map_err(|error| StoreError::Unavailable(format!("keyring set: {error}")))?;
        match entry.get_password() {
            Ok(value) if value == secret => Ok(()),
            Ok(_) => Err(StoreError::Unavailable(
                "keyring verify failed: password mismatch after set".into(),
            )),
            Err(error) => Err(StoreError::Unavailable(format!(
                "keyring verify failed after set: {error}"
            ))),
        }
    }

    fn keyring_get(account_id: &str) -> Result<Option<String>, StoreError> {
        match Self::keyring_entry(account_id)?.get_password() {
            Ok(value) if value.is_empty() => Ok(None),
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(StoreError::Unavailable(format!("keyring get: {error}"))),
        }
    }
}

impl MailStore for SqliteMailStore {
    fn list_templates(&self) -> Result<Vec<MailTemplateRecord>, StoreError> {
        self.db.with_conn(|conn| {
            template::mail_template
                .order((template::sort_order.asc(), template::created_at.asc()))
                .select(MailTemplateRow::as_select())
                .load::<MailTemplateRow>(conn)
                .map(|rows| rows.into_iter().map(MailTemplateRecord::from).collect())
                .map_err(map_diesel_error)
        })
    }

    fn get_template(&self, id: &str) -> Result<MailTemplateRecord, StoreError> {
        self.db.with_conn(|conn| {
            template::mail_template
                .filter(template::id.eq(id))
                .select(MailTemplateRow::as_select())
                .first::<MailTemplateRow>(conn)
                .map(MailTemplateRecord::from)
                .map_err(map_diesel_error)
        })
    }

    fn save_template(
        &self,
        input: MailTemplateWriteInput,
    ) -> Result<MailTemplateRecord, StoreError> {
        let now = now_string();
        let id = input
            .id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        self.db.with_conn(|conn| {
            if let Some(existing_id) = input.id.as_deref() {
                let existing = template::mail_template
                    .filter(template::id.eq(existing_id))
                    .select(MailTemplateRow::as_select())
                    .first::<MailTemplateRow>(conn)
                    .map_err(map_diesel_error)?;
                if existing.is_system {
                    return Err(StoreError::Conflict(
                        "mail.template.system_readonly".to_string(),
                    ));
                }

                diesel::update(template::mail_template.filter(template::id.eq(existing_id)))
                    .set((
                        template::name.eq(&input.name),
                        template::template_intent.eq(&input.template_intent),
                        template::subject_template.eq(&input.subject_template),
                        template::body_text_template.eq(&input.body_text_template),
                        template::body_html_template.eq(&input.body_html_template),
                        template::locale.eq(&input.locale),
                        template::is_active.eq(input.is_active),
                        template::sort_order.eq(input.sort_order),
                        template::updated_at.eq(&now),
                    ))
                    .execute(conn)
                    .map_err(map_diesel_error)?;
            } else {
                let row = NewMailTemplateRow {
                    id: id.clone(),
                    name: input.name.clone(),
                    template_intent: input.template_intent.clone(),
                    subject_template: input.subject_template.clone(),
                    body_text_template: input.body_text_template.clone(),
                    body_html_template: input.body_html_template.clone(),
                    locale: input.locale.clone(),
                    is_system: false,
                    is_active: input.is_active,
                    sort_order: input.sort_order,
                    created_at: now.clone(),
                    updated_at: now.clone(),
                };
                diesel::insert_into(template::mail_template)
                    .values(&row)
                    .execute(conn)
                    .map_err(map_diesel_error)?;
            }

            template::mail_template
                .filter(template::id.eq(&id))
                .select(MailTemplateRow::as_select())
                .first::<MailTemplateRow>(conn)
                .map(MailTemplateRecord::from)
                .map_err(map_diesel_error)
        })
    }

    fn list_accounts(&self) -> Result<Vec<MailAccountRecord>, StoreError> {
        self.db.with_conn(|conn| {
            account::mail_account
                .order(account::updated_at.desc())
                .select(MailAccountRow::as_select())
                .load::<MailAccountRow>(conn)
                .map(|rows| rows.into_iter().map(MailAccountRecord::from).collect())
                .map_err(map_diesel_error)
        })
    }

    fn get_account(&self, id: &str) -> Result<MailAccountRecord, StoreError> {
        self.db.with_conn(|conn| {
            account::mail_account
                .filter(account::id.eq(id))
                .select(MailAccountRow::as_select())
                .first::<MailAccountRow>(conn)
                .map(MailAccountRecord::from)
                .map_err(map_diesel_error)
        })
    }

    fn save_account(&self, input: MailAccountWriteInput) -> Result<MailAccountRecord, StoreError> {
        let id = input
            .id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let now = now_string();
        let is_update = input.id.is_some();
        let password = input.password.trim();

        if password.is_empty() && !is_update {
            return Err(StoreError::Unavailable(
                "mail.account.password_required".to_string(),
            ));
        }

        if !password.is_empty() {
            Self::keyring_set(&id, password)?;
        } else if Self::keyring_get(&id)?.is_none() {
            // Legacy rows may still hold inline password_value; migrate on keep-password update.
            let legacy = self.get_account(&id)?;
            if !legacy.password_value.is_empty() {
                Self::keyring_set(&id, &legacy.password_value)?;
            } else {
                return Err(StoreError::Unavailable(
                    "mail.account.password_required".to_string(),
                ));
            }
        }

        let row = NewMailAccountRow {
            id: id.clone(),
            label: input.label,
            from_address: normalize_email(&input.from_address),
            from_name: input.from_name,
            smtp_host: input.smtp_host,
            smtp_port: input.smtp_port,
            use_tls: input.use_tls,
            username: input.username,
            password_ref: mail_password_ref(&id),
            password_value: String::new(),
            imap_host: input.imap_host,
            imap_port: input.imap_port,
            imap_use_tls: input.imap_use_tls,
            imap_sync_enabled: input.imap_sync_enabled,
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        self.db.with_conn(|conn| {
            let exists = account::mail_account
                .filter(account::id.eq(&id))
                .count()
                .get_result::<i64>(conn)
                .map_err(map_diesel_error)?
                > 0;

            if exists {
                diesel::update(account::mail_account.filter(account::id.eq(&id)))
                    .set((
                        account::label.eq(&row.label),
                        account::from_address.eq(&row.from_address),
                        account::from_name.eq(&row.from_name),
                        account::smtp_host.eq(&row.smtp_host),
                        account::smtp_port.eq(row.smtp_port),
                        account::use_tls.eq(row.use_tls),
                        account::username.eq(&row.username),
                        account::password_ref.eq(&row.password_ref),
                        account::password_value.eq(&row.password_value),
                        account::imap_host.eq(&row.imap_host),
                        account::imap_port.eq(row.imap_port),
                        account::imap_use_tls.eq(row.imap_use_tls),
                        account::imap_sync_enabled.eq(row.imap_sync_enabled),
                        account::updated_at.eq(&row.updated_at),
                    ))
                    .execute(conn)
                    .map_err(map_diesel_error)?;
            } else {
                diesel::insert_into(account::mail_account)
                    .values(&row)
                    .execute(conn)
                    .map_err(map_diesel_error)?;
            }

            account::mail_account
                .filter(account::id.eq(&id))
                .select(MailAccountRow::as_select())
                .first::<MailAccountRow>(conn)
                .map(MailAccountRecord::from)
                .map_err(map_diesel_error)
        })
    }

    fn resolve_account_password(&self, account_id: &str) -> Result<String, StoreError> {
        if let Some(secret) = Self::keyring_get(account_id)? {
            return Ok(secret);
        }

        let account = self.get_account(account_id)?;
        if !account.password_value.is_empty() {
            // Migrate legacy inline secret into keyring on first read.
            Self::keyring_set(account_id, &account.password_value)?;
            let _ = self.db.with_conn(|conn| {
                diesel::update(account::mail_account.filter(account::id.eq(account_id)))
                    .set((
                        account::password_ref.eq(mail_password_ref(account_id)),
                        account::password_value.eq(""),
                    ))
                    .execute(conn)
                    .map_err(map_diesel_error)
            });
            return Ok(account.password_value);
        }

        Err(StoreError::Unavailable(
            "mail.account.password_missing".to_string(),
        ))
    }

    fn create_outbound_message(
        &self,
        input: MailSendInput,
    ) -> Result<MailMessageRecord, StoreError> {
        let now = now_string();
        let row = NewMailMessageRow {
            id: Uuid::new_v4().to_string(),
            customer_id: input.customer_id.clone(),
            template_id: input.template_id.clone(),
            account_id: Some(input.account_id.clone()),
            status: input.status.clone(),
            direction: "outbound".to_string(),
            subject: input.subject.clone(),
            body_text: input.body_text.clone(),
            body_html: input.body_html,
            error_message: input.error_message.clone(),
            sent_at: input.sent_at,
            received_at: None,
            imap_uid: None,
            imap_folder: None,
            rfc_message_id: input.rfc_message_id,
            in_reply_to: None,
            references_header: None,
            is_favorite: false,
            open_tracking_id: input.open_tracking_id,
            created_at: now.clone(),
            updated_at: now.clone(),
            to_address: Some(input.to_address.clone()),
            from_address: Some(input.from_address.clone()),
            source_ref: None,
        };

        let summary = if input.status == "sent" {
            format!(
                "Email sent [{}] -> {}: {}",
                input.template_name, input.to_address, input.subject
            )
        } else {
            format!(
                "Email failed [{}] -> {}: {}",
                input.template_name, input.to_address, input.subject
            )
        };

        self.db.with_conn(|conn| {
            diesel::insert_into(message::mail_message)
                .values(&row)
                .execute(conn)
                .map_err(map_diesel_error)?;

            if let Some(customer_id) = input.customer_id.as_deref() {
                insert_timeline_entry(conn, customer_id, "email_sent", &row.id, &summary)?;
            }

            message::mail_message
                .filter(message::id.eq(&row.id))
                .select(MailMessageRow::as_select())
                .first::<MailMessageRow>(conn)
                .map(MailMessageRecord::from)
                .map_err(map_diesel_error)
        })
    }

    fn create_inbound_message(
        &self,
        input: MailInboundWriteInput,
    ) -> Result<MailMessageRecord, StoreError> {
        let now = now_string();
        let row = NewMailMessageRow {
            id: Uuid::new_v4().to_string(),
            customer_id: Some(input.customer_id.clone()),
            template_id: None,
            account_id: None,
            status: "received".to_string(),
            direction: "inbound".to_string(),
            subject: input.subject.clone(),
            body_text: input.body_text.clone(),
            body_html: input.body_html,
            error_message: None,
            sent_at: None,
            received_at: Some(input.received_at),
            imap_uid: None,
            imap_folder: None,
            rfc_message_id: input.rfc_message_id,
            in_reply_to: input.in_reply_to,
            references_header: Some(input.from_address.clone()),
            is_favorite: false,
            open_tracking_id: None,
            created_at: now.clone(),
            updated_at: now.clone(),
            to_address: Some(input.from_address.clone()),
            from_address: Some(input.from_address.clone()),
            source_ref: None,
        };

        self.db.with_conn(|conn| {
            diesel::insert_into(message::mail_message)
                .values(&row)
                .execute(conn)
                .map_err(map_diesel_error)?;

            insert_timeline_entry(
                conn,
                &input.customer_id,
                "email_received",
                &row.id,
                &format!("Email received: {}", row.subject),
            )?;

            message::mail_message
                .filter(message::id.eq(&row.id))
                .select(MailMessageRow::as_select())
                .first::<MailMessageRow>(conn)
                .map(MailMessageRecord::from)
                .map_err(map_diesel_error)
        })
    }

    fn list_messages(
        &self,
        filter: MailMessageListFilter,
    ) -> Result<(Vec<MailMessageRecord>, i64), StoreError> {
        let direction = filter.direction.trim().to_ascii_lowercase();
        if direction != "inbound" && direction != "outbound" {
            return Err(StoreError::Unavailable(
                "mail.message.direction_invalid".to_string(),
            ));
        }
        let limit = filter.limit.clamp(1, 500);
        let offset = filter.offset.max(0);
        let query = filter
            .query
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| format!("%{value}%"));

        self.db.with_conn(|conn| {
            let mut count_query = message::mail_message
                .filter(message::direction.eq(&direction))
                .into_boxed();
            let mut rows_query = message::mail_message
                .filter(message::direction.eq(&direction))
                .into_boxed();

            if let Some(account_id) = filter.account_id.as_deref() {
                count_query = count_query.filter(message::account_id.eq(account_id));
                rows_query = rows_query.filter(message::account_id.eq(account_id));
            }
            if let Some(customer_id) = filter.customer_id.as_deref() {
                count_query = count_query.filter(message::customer_id.eq(customer_id));
                rows_query = rows_query.filter(message::customer_id.eq(customer_id));
            }
            if let Some(pattern) = query.as_deref() {
                count_query = count_query.filter(
                    message::subject
                        .like(pattern)
                        .or(message::body_text.like(pattern)),
                );
                rows_query = rows_query.filter(
                    message::subject
                        .like(pattern)
                        .or(message::body_text.like(pattern)),
                );
            }

            let total = count_query
                .count()
                .get_result::<i64>(conn)
                .map_err(map_diesel_error)?;

            let rows = rows_query
                .order(message::created_at.desc())
                .limit(limit)
                .offset(offset)
                .select(MailMessageRow::as_select())
                .load::<MailMessageRow>(conn)
                .map_err(map_diesel_error)?;

            Ok((
                rows.into_iter().map(MailMessageRecord::from).collect(),
                total,
            ))
        })
    }
}

fn insert_timeline_entry(
    conn: &mut diesel::sqlite::SqliteConnection,
    customer_id: &str,
    entry_type: &str,
    ref_id: &str,
    summary: &str,
) -> Result<(), StoreError> {
    diesel::insert_into(timeline::customer_timeline)
        .values((
            timeline::id.eq(Uuid::new_v4().to_string()),
            timeline::customer_id.eq(customer_id),
            timeline::entry_type.eq(entry_type),
            timeline::ref_id.eq(Some(ref_id.to_string())),
            timeline::summary.eq(summary.to_string()),
            timeline::metadata_json.eq::<Option<String>>(None),
            timeline::created_at.eq(now_string()),
        ))
        .execute(conn)
        .map(|_| ())
        .map_err(map_diesel_error)
}

impl From<MailTemplateRow> for MailTemplateRecord {
    fn from(row: MailTemplateRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            template_intent: row.template_intent,
            subject_template: row.subject_template,
            body_text_template: row.body_text_template,
            body_html_template: row.body_html_template,
            locale: row.locale,
            is_system: row.is_system,
            is_active: row.is_active,
            sort_order: row.sort_order,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<MailAccountRow> for MailAccountRecord {
    fn from(row: MailAccountRow) -> Self {
        Self {
            id: row.id,
            label: row.label,
            from_address: row.from_address,
            from_name: row.from_name,
            smtp_host: row.smtp_host,
            smtp_port: row.smtp_port,
            use_tls: row.use_tls,
            username: row.username,
            password_ref: row.password_ref,
            password_value: row.password_value,
            imap_host: row.imap_host,
            imap_port: row.imap_port,
            imap_use_tls: row.imap_use_tls,
            imap_sync_enabled: row.imap_sync_enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<MailMessageRow> for MailMessageRecord {
    fn from(row: MailMessageRow) -> Self {
        Self {
            id: row.id,
            customer_id: row.customer_id,
            template_id: row.template_id,
            account_id: row.account_id,
            status: row.status,
            direction: row.direction,
            subject: row.subject,
            body_text: row.body_text,
            body_html: row.body_html,
            to_address: row.to_address,
            from_address: row.from_address,
            error_message: row.error_message,
            sent_at: row.sent_at,
            received_at: row.received_at,
            imap_uid: row.imap_uid,
            imap_folder: row.imap_folder,
            rfc_message_id: row.rfc_message_id,
            in_reply_to: row.in_reply_to,
            references: row.references_header,
            is_favorite: row.is_favorite,
            open_tracking_id: row.open_tracking_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

fn normalize_email(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}

fn now_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("{millis}")
}

fn map_diesel_error(error: diesel::result::Error) -> StoreError {
    match error {
        diesel::result::Error::NotFound => StoreError::NotFound,
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            _,
        ) => StoreError::Conflict("mail.message_duplicate".to_string()),
        other => StoreError::Unavailable(other.to_string()),
    }
}

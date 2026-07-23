//! Diesel-backed `customer` store.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use diesel::prelude::*;
use ports::customer::{
    CustomerListQuery, CustomerListResult, CustomerRecord, CustomerStore, CustomerWriteInput,
};
use ports::repository::StoreError;
use uuid::Uuid;

use crate::opendesk_db::schema::customer::dsl as customer;
use crate::opendesk_db::{CustomerRow, NewCustomerRow, OpendeskDb};

/// SQLite implementation of [`CustomerStore`].
pub struct SqliteCustomerStore {
    db: OpendeskDb,
}

impl SqliteCustomerStore {
    /// Open `opendesk.db` and return a customer store handle.
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-21
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        Ok(Self {
            db: OpendeskDb::open(path)?,
        })
    }

    /// Wrap an existing [`OpendeskDb`] handle.
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-21
    pub fn new(db: OpendeskDb) -> Self {
        Self { db }
    }
}

impl CustomerStore for SqliteCustomerStore {
    fn list(&self, query: CustomerListQuery) -> Result<CustomerListResult, StoreError> {
        self.db.with_conn(|conn| {
            let search = query
                .search
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| format!("%{value}%"));

            let build_query = || {
                let mut list_query = customer::customer.into_boxed();
                if let Some(pattern) = search.as_deref() {
                    list_query = list_query.filter(
                        customer::email
                            .like(pattern)
                            .or(customer::display_name.like(pattern)),
                    );
                }
                list_query
            };

            let total = build_query()
                .count()
                .get_result::<i64>(conn)
                .map_err(map_diesel_error)?;

            let rows = build_query()
                .order(customer::updated_at.desc())
                .limit(query.limit)
                .offset(query.offset)
                .select(CustomerRow::as_select())
                .load::<CustomerRow>(conn)
                .map_err(map_diesel_error)?;

            Ok(CustomerListResult {
                items: rows.into_iter().map(CustomerRecord::from).collect(),
                total,
            })
        })
    }

    fn get(&self, id: &str) -> Result<CustomerRecord, StoreError> {
        self.db.with_conn(|conn| {
            customer::customer
                .filter(customer::id.eq(id))
                .select(CustomerRow::as_select())
                .first::<CustomerRow>(conn)
                .optional()
                .map_err(map_diesel_error)?
                .map(CustomerRecord::from)
                .ok_or(StoreError::NotFound)
        })
    }

    fn create(&self, input: CustomerWriteInput) -> Result<CustomerRecord, StoreError> {
        let now = now_string();
        let row = NewCustomerRow {
            id: Uuid::new_v4().to_string(),
            display_name: input.display_name,
            email: normalize_email(&input.email),
            whatsapp_phone: input.whatsapp_phone,
            source_channel: input.source_channel,
            source_meta: input.source_meta,
            lifecycle_status: input.lifecycle_status,
            outreach_stage: input.outreach_stage,
            quoted_price: input.quoted_price.map(|value| value as f32),
            quoted_currency: input.quoted_currency,
            quoted_at: input.quoted_at,
            pricing_tier: input.pricing_tier,
            cooperation_status: input.cooperation_status,
            package_name: input.package_name,
            monthly_fee: input.monthly_fee.map(|value| value as f32),
            contract_start: input.contract_start,
            contract_end: input.contract_end,
            notes: input.notes,
            created_at: now.clone(),
            updated_at: now,
            extra_json: None,
            source_ref: None,
        };

        self.db.with_conn(|conn| {
            diesel::insert_into(customer::customer)
                .values(&row)
                .execute(conn)
                .map_err(map_diesel_error)?;
            customer::customer
                .filter(customer::id.eq(&row.id))
                .select(CustomerRow::as_select())
                .first::<CustomerRow>(conn)
                .map(CustomerRecord::from)
                .map_err(map_diesel_error)
        })
    }

    fn update(&self, id: &str, input: CustomerWriteInput) -> Result<CustomerRecord, StoreError> {
        let now = now_string();
        self.db.with_conn(|conn| {
            let updated = diesel::update(customer::customer.filter(customer::id.eq(id)))
                .set((
                    customer::display_name.eq(input.display_name),
                    customer::email.eq(normalize_email(&input.email)),
                    customer::whatsapp_phone.eq(input.whatsapp_phone),
                    customer::source_channel.eq(input.source_channel),
                    customer::source_meta.eq(input.source_meta),
                    customer::lifecycle_status.eq(input.lifecycle_status),
                    customer::outreach_stage.eq(input.outreach_stage),
                    customer::quoted_price.eq(input.quoted_price.map(|value| value as f32)),
                    customer::quoted_currency.eq(input.quoted_currency),
                    customer::quoted_at.eq(input.quoted_at),
                    customer::pricing_tier.eq(input.pricing_tier),
                    customer::cooperation_status.eq(input.cooperation_status),
                    customer::package_name.eq(input.package_name),
                    customer::monthly_fee.eq(input.monthly_fee.map(|value| value as f32)),
                    customer::contract_start.eq(input.contract_start),
                    customer::contract_end.eq(input.contract_end),
                    customer::notes.eq(input.notes),
                    customer::updated_at.eq(now),
                ))
                .execute(conn)
                .map_err(map_diesel_error)?;

            if updated == 0 {
                return Err(StoreError::NotFound);
            }

            customer::customer
                .filter(customer::id.eq(id))
                .select(CustomerRow::as_select())
                .first::<CustomerRow>(conn)
                .map(CustomerRecord::from)
                .map_err(map_diesel_error)
        })
    }

    fn find_by_email(&self, email: &str) -> Result<Option<CustomerRecord>, StoreError> {
        let normalized = normalize_email(email);
        if normalized.is_empty() {
            return Ok(None);
        }
        self.db.with_conn(|conn| {
            customer::customer
                .filter(customer::email.eq(normalized))
                .select(CustomerRow::as_select())
                .first::<CustomerRow>(conn)
                .optional()
                .map(|row| row.map(CustomerRecord::from))
                .map_err(map_diesel_error)
        })
    }
}

impl From<CustomerRow> for CustomerRecord {
    fn from(row: CustomerRow) -> Self {
        Self {
            id: row.id,
            display_name: row.display_name,
            email: row.email,
            whatsapp_phone: row.whatsapp_phone,
            source_channel: row.source_channel,
            source_meta: row.source_meta,
            lifecycle_status: row.lifecycle_status,
            outreach_stage: row.outreach_stage,
            quoted_price: row.quoted_price.map(f64::from),
            quoted_currency: row.quoted_currency,
            quoted_at: row.quoted_at,
            pricing_tier: row.pricing_tier,
            cooperation_status: row.cooperation_status,
            package_name: row.package_name,
            monthly_fee: row.monthly_fee.map(f64::from),
            contract_start: row.contract_start,
            contract_end: row.contract_end,
            notes: row.notes,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

fn normalize_email(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}

fn now_string() -> String {
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
        ) => StoreError::Conflict("customer.email_duplicate".to_string()),
        other => StoreError::Unavailable(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ports::customer::CustomerWriteInput;

    fn sample_input(email: &str) -> CustomerWriteInput {
        CustomerWriteInput {
            display_name: Some("Acme".to_string()),
            email: email.to_string(),
            whatsapp_phone: None,
            source_channel: "manual".to_string(),
            source_meta: None,
            lifecycle_status: "new".to_string(),
            outreach_stage: "no_stage".to_string(),
            quoted_price: None,
            quoted_currency: None,
            quoted_at: None,
            pricing_tier: None,
            cooperation_status: "none".to_string(),
            package_name: None,
            monthly_fee: None,
            contract_start: None,
            contract_end: None,
            notes: None,
        }
    }

    #[test]
    fn create_rejects_duplicate_email() {
        let dir = std::env::temp_dir().join(format!("customer_store_test_{}", Uuid::new_v4()));
        let path = dir.join("test.db");
        let store = SqliteCustomerStore::open(&path).expect("open");
        store
            .create(sample_input("dup@example.com"))
            .expect("first create");
        let error = store
            .create(sample_input("dup@example.com"))
            .expect_err("duplicate");
        assert!(matches!(error, StoreError::Conflict(_)));
        let _ = std::fs::remove_dir_all(dir);
    }
}

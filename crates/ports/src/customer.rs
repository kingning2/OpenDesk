//! Customer profile persistence port.

use crate::repository::StoreError;

/// Customer list query parameters.
#[derive(Debug, Clone)]
pub struct CustomerListQuery {
    pub search: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

/// Customer list result.
#[derive(Debug, Clone)]
pub struct CustomerListResult {
    pub items: Vec<CustomerRecord>,
    pub total: i64,
}

/// Writable customer fields shared by create and update.
#[derive(Debug, Clone)]
pub struct CustomerWriteInput {
    pub display_name: Option<String>,
    pub email: String,
    pub whatsapp_phone: Option<String>,
    pub source_channel: String,
    pub source_meta: Option<String>,
    pub lifecycle_status: String,
    pub outreach_stage: String,
    pub quoted_price: Option<f64>,
    pub quoted_currency: Option<String>,
    pub quoted_at: Option<String>,
    pub pricing_tier: Option<String>,
    pub cooperation_status: String,
    pub package_name: Option<String>,
    pub monthly_fee: Option<f64>,
    pub contract_start: Option<String>,
    pub contract_end: Option<String>,
    pub notes: Option<String>,
}

/// Customer profile record from SQLite.
#[derive(Debug, Clone)]
pub struct CustomerRecord {
    pub id: String,
    pub display_name: Option<String>,
    pub email: String,
    pub whatsapp_phone: Option<String>,
    pub source_channel: String,
    pub source_meta: Option<String>,
    pub lifecycle_status: String,
    pub outreach_stage: String,
    pub quoted_price: Option<f64>,
    pub quoted_currency: Option<String>,
    pub quoted_at: Option<String>,
    pub pricing_tier: Option<String>,
    pub cooperation_status: String,
    pub package_name: Option<String>,
    pub monthly_fee: Option<f64>,
    pub contract_start: Option<String>,
    pub contract_end: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Customer profile store port.
pub trait CustomerStore: Send + Sync {
    fn list(&self, query: CustomerListQuery) -> Result<CustomerListResult, StoreError>;

    fn get(&self, id: &str) -> Result<CustomerRecord, StoreError>;

    fn create(&self, input: CustomerWriteInput) -> Result<CustomerRecord, StoreError>;

    fn update(&self, id: &str, input: CustomerWriteInput) -> Result<CustomerRecord, StoreError>;

    /// Find one customer by exact email (case-insensitive).
    fn find_by_email(&self, email: &str) -> Result<Option<CustomerRecord>, StoreError>;
}

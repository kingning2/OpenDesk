use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerDtoProfile {
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

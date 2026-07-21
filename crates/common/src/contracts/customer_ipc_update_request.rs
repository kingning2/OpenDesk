use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerIpcUpdateRequest {
    pub trace_id: Option<String>,
    pub id: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub whatsapp_phone: Option<String>,
    pub source_channel: Option<String>,
    pub source_meta: Option<String>,
    pub lifecycle_status: Option<String>,
    pub outreach_stage: Option<String>,
    pub quoted_price: Option<f64>,
    pub quoted_currency: Option<String>,
    pub quoted_at: Option<String>,
    pub pricing_tier: Option<String>,
    pub cooperation_status: Option<String>,
    pub package_name: Option<String>,
    pub monthly_fee: Option<f64>,
    pub contract_start: Option<String>,
    pub contract_end: Option<String>,
    pub notes: Option<String>,
}

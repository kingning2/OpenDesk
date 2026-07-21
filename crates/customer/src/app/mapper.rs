//! Map domain records to contract DTO JSON.

use common::contracts::CustomerDtoProfile;
use ports::customer::CustomerRecord;

/// Serialize a customer record to contract profile JSON.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
///
/// # 参数
/// - `record` — SQLite customer row mapped to port record
///
/// # 返回值
/// JSON string matching [`CustomerDtoProfile`].
pub fn record_to_profile_json(record: &CustomerRecord) -> Result<String, String> {
    let dto = CustomerDtoProfile {
        id: record.id.clone(),
        display_name: record.display_name.clone(),
        email: record.email.clone(),
        whatsapp_phone: record.whatsapp_phone.clone(),
        source_channel: record.source_channel.clone(),
        source_meta: record.source_meta.clone(),
        lifecycle_status: record.lifecycle_status.clone(),
        outreach_stage: record.outreach_stage.clone(),
        quoted_price: record.quoted_price,
        quoted_currency: record.quoted_currency.clone(),
        quoted_at: record.quoted_at.clone(),
        pricing_tier: record.pricing_tier.clone(),
        cooperation_status: record.cooperation_status.clone(),
        package_name: record.package_name.clone(),
        monthly_fee: record.monthly_fee,
        contract_start: record.contract_start.clone(),
        contract_end: record.contract_end.clone(),
        notes: record.notes.clone(),
        created_at: record.created_at.clone(),
        updated_at: record.updated_at.clone(),
    };
    serde_json::to_string(&dto).map_err(|error| error.to_string())
}

/// Serialize multiple customer records to a JSON array string.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
pub fn records_to_profiles_json(records: &[CustomerRecord]) -> Result<String, String> {
    let profiles: Result<Vec<CustomerDtoProfile>, String> = records
        .iter()
        .map(|record| {
            Ok(CustomerDtoProfile {
                id: record.id.clone(),
                display_name: record.display_name.clone(),
                email: record.email.clone(),
                whatsapp_phone: record.whatsapp_phone.clone(),
                source_channel: record.source_channel.clone(),
                source_meta: record.source_meta.clone(),
                lifecycle_status: record.lifecycle_status.clone(),
                outreach_stage: record.outreach_stage.clone(),
                quoted_price: record.quoted_price,
                quoted_currency: record.quoted_currency.clone(),
                quoted_at: record.quoted_at.clone(),
                pricing_tier: record.pricing_tier.clone(),
                cooperation_status: record.cooperation_status.clone(),
                package_name: record.package_name.clone(),
                monthly_fee: record.monthly_fee,
                contract_start: record.contract_start.clone(),
                contract_end: record.contract_end.clone(),
                notes: record.notes.clone(),
                created_at: record.created_at.clone(),
                updated_at: record.updated_at.clone(),
            })
        })
        .collect();
    serde_json::to_string(&profiles?).map_err(|error| error.to_string())
}

//! Update an existing customer profile.

use common::contracts::{CustomerIpcUpdateRequest, CustomerIpcUpdateResponse};
use ports::customer::{CustomerRecord, CustomerStore, CustomerWriteInput};
use ports::repository::StoreError;

use super::mapper::record_to_profile_json;

/// Update customer use case.
pub struct UpdateCustomer;

impl UpdateCustomer {
    /// Update a customer profile in the store.
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-21
    pub fn execute<S: CustomerStore + ?Sized>(
        store: &S,
        request: CustomerIpcUpdateRequest,
    ) -> Result<CustomerIpcUpdateResponse, String> {
        let existing = store.get(&request.id).map_err(map_store_error)?;
        let input = merge_update(&existing, &request);
        let record = store.update(&request.id, input).map_err(map_store_error)?;
        Ok(CustomerIpcUpdateResponse {
            ok: true,
            profile_json: record_to_profile_json(&record)?,
            trace_id: request.trace_id,
        })
    }
}

fn merge_update(
    existing: &CustomerRecord,
    request: &CustomerIpcUpdateRequest,
) -> CustomerWriteInput {
    CustomerWriteInput {
        display_name: request
            .display_name
            .clone()
            .or(existing.display_name.clone()),
        email: request
            .email
            .clone()
            .unwrap_or_else(|| existing.email.clone()),
        whatsapp_phone: request
            .whatsapp_phone
            .clone()
            .or(existing.whatsapp_phone.clone()),
        source_channel: request
            .source_channel
            .clone()
            .unwrap_or_else(|| existing.source_channel.clone()),
        source_meta: request.source_meta.clone().or(existing.source_meta.clone()),
        lifecycle_status: request
            .lifecycle_status
            .clone()
            .unwrap_or_else(|| existing.lifecycle_status.clone()),
        outreach_stage: request
            .outreach_stage
            .clone()
            .unwrap_or_else(|| existing.outreach_stage.clone()),
        quoted_price: request.quoted_price.or(existing.quoted_price),
        quoted_currency: request
            .quoted_currency
            .clone()
            .or(existing.quoted_currency.clone()),
        quoted_at: request.quoted_at.clone().or(existing.quoted_at.clone()),
        pricing_tier: request
            .pricing_tier
            .clone()
            .or(existing.pricing_tier.clone()),
        cooperation_status: request
            .cooperation_status
            .clone()
            .unwrap_or_else(|| existing.cooperation_status.clone()),
        package_name: request
            .package_name
            .clone()
            .or(existing.package_name.clone()),
        monthly_fee: request.monthly_fee.or(existing.monthly_fee),
        contract_start: request
            .contract_start
            .clone()
            .or(existing.contract_start.clone()),
        contract_end: request
            .contract_end
            .clone()
            .or(existing.contract_end.clone()),
        notes: request.notes.clone().or(existing.notes.clone()),
    }
}

fn map_store_error(error: StoreError) -> String {
    match error {
        StoreError::NotFound => "customer.not_found".to_string(),
        StoreError::Conflict(code) => code,
        StoreError::Unavailable(message) => message,
    }
}

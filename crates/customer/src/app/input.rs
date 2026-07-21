//! Shared input mapping for customer write operations.

use common::contracts::CustomerIpcCreateRequest;
use ports::customer::CustomerWriteInput;

const DEFAULT_SOURCE_CHANNEL: &str = "manual";
const DEFAULT_LIFECYCLE_STATUS: &str = "new";
const DEFAULT_OUTREACH_STAGE: &str = "no_stage";
const DEFAULT_COOPERATION_STATUS: &str = "none";

/// Map create IPC request to store write input.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
pub fn write_input_from_create(request: &CustomerIpcCreateRequest) -> CustomerWriteInput {
    CustomerWriteInput {
        display_name: request.display_name.clone(),
        email: request.email.clone(),
        whatsapp_phone: request.whatsapp_phone.clone(),
        source_channel: request
            .source_channel
            .clone()
            .unwrap_or_else(|| DEFAULT_SOURCE_CHANNEL.to_string()),
        source_meta: request.source_meta.clone(),
        lifecycle_status: request
            .lifecycle_status
            .clone()
            .unwrap_or_else(|| DEFAULT_LIFECYCLE_STATUS.to_string()),
        outreach_stage: request
            .outreach_stage
            .clone()
            .unwrap_or_else(|| DEFAULT_OUTREACH_STAGE.to_string()),
        quoted_price: request.quoted_price,
        quoted_currency: request.quoted_currency.clone(),
        quoted_at: request.quoted_at.clone(),
        pricing_tier: request.pricing_tier.clone(),
        cooperation_status: request
            .cooperation_status
            .clone()
            .unwrap_or_else(|| DEFAULT_COOPERATION_STATUS.to_string()),
        package_name: request.package_name.clone(),
        monthly_fee: request.monthly_fee,
        contract_start: request.contract_start.clone(),
        contract_end: request.contract_end.clone(),
        notes: request.notes.clone(),
    }
}

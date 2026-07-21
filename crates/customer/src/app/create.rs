//! Create a new customer profile.

use common::contracts::{CustomerIpcCreateRequest, CustomerIpcCreateResponse};
use ports::customer::CustomerStore;
use ports::repository::StoreError;

use super::input::write_input_from_create;
use super::mapper::record_to_profile_json;

/// Create customer use case.
pub struct CreateCustomer;

impl CreateCustomer {
    /// Create a customer profile in the store.
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-21
    pub fn execute<S: CustomerStore + ?Sized>(
        store: &S,
        request: CustomerIpcCreateRequest,
    ) -> Result<CustomerIpcCreateResponse, String> {
        let email = request.email.trim();
        if email.is_empty() {
            return Err("customer.email_required".to_string());
        }

        let input = write_input_from_create(&request);
        let record = store.create(input).map_err(map_store_error)?;
        Ok(CustomerIpcCreateResponse {
            ok: true,
            profile_json: record_to_profile_json(&record)?,
            trace_id: request.trace_id,
        })
    }
}

fn map_store_error(error: StoreError) -> String {
    match error {
        StoreError::NotFound => "customer.not_found".to_string(),
        StoreError::Conflict(code) => code,
        StoreError::Unavailable(message) => message,
    }
}

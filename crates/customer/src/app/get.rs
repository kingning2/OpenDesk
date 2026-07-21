//! Fetch a single customer profile.

use common::contracts::{CustomerIpcGetRequest, CustomerIpcGetResponse};
use ports::customer::CustomerStore;
use ports::repository::StoreError;

use super::mapper::record_to_profile_json;

/// Get one customer by id.
pub struct GetCustomer;

impl GetCustomer {
    /// Execute get query against the customer store.
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-21
    pub fn execute<S: CustomerStore + ?Sized>(
        store: &S,
        request: CustomerIpcGetRequest,
    ) -> Result<CustomerIpcGetResponse, String> {
        let record = store.get(&request.id).map_err(map_store_error)?;
        Ok(CustomerIpcGetResponse {
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

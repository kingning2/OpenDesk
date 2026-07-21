//! List customers with optional search and pagination.

use common::contracts::{CustomerIpcListRequest, CustomerIpcListResponse};
use ports::customer::{CustomerListQuery, CustomerStore};
use ports::repository::StoreError;

use super::mapper::records_to_profiles_json;

const DEFAULT_LIMIT: i64 = 50;

/// List customer profiles.
pub struct ListCustomers;

impl ListCustomers {
    /// Execute list query against the customer store.
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-21
    pub fn execute<S: CustomerStore + ?Sized>(
        store: &S,
        request: CustomerIpcListRequest,
    ) -> Result<CustomerIpcListResponse, String> {
        let limit = request.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, 200);
        let offset = request.offset.unwrap_or(0).max(0);
        let result = store
            .list(CustomerListQuery {
                search: request.search,
                limit,
                offset,
            })
            .map_err(map_store_error)?;
        Ok(CustomerIpcListResponse {
            ok: true,
            customers_json: records_to_profiles_json(&result.items)?,
            total: result.total,
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

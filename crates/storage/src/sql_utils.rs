//! 存储层 Diesel 错误映射工具。

use ports::repository::StoreError;

/// 将 Diesel 错误映射为 [`StoreError`]。
///
/// `conflict_msg` 为 `Some` 时，唯一键冲突映射为 `Conflict`（各表用不同文案）；
/// 为 `None` 时保持旧行为，唯一键冲突落到 `Unavailable`。
pub fn map_diesel_error(error: diesel::result::Error, conflict_msg: Option<&str>) -> StoreError {
    if let diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UniqueViolation,
        _,
    ) = &error
    {
        if let Some(message) = conflict_msg {
            return StoreError::Conflict(message.to_string());
        }
    }
    match error {
        diesel::result::Error::NotFound => StoreError::NotFound,
        other => StoreError::Unavailable(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_maps_to_not_found() {
        assert!(matches!(
            map_diesel_error(diesel::result::Error::NotFound, Some("x")),
            StoreError::NotFound
        ));
    }
}

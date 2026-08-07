//! Unix 时间戳工具：消除各 crate 内重复的 epoch 计算。

use std::time::{SystemTime, UNIX_EPOCH};

/// 当前 Unix 秒（字符串）。
pub fn now_secs_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

/// 当前 Unix 毫秒（字符串）。
pub fn now_millis_string() -> String {
    now_millis_u128().to_string()
}

/// 当前 Unix 毫秒（`i64`）。
pub fn now_millis_i64() -> i64 {
    now_millis_u128() as i64
}

fn now_millis_u128() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers_return_plausible_epoch_values() {
        let secs = now_secs_string();
        let millis = now_millis_string();
        let millis_i64 = now_millis_i64();
        assert!(secs.parse::<u64>().unwrap() > 1_700_000_000);
        assert!(millis.parse::<u64>().unwrap() > 1_700_000_000_000);
        assert!(millis_i64 > 1_700_000_000_000);
        // 毫秒串与 i64 同源。
        assert_eq!(millis, millis_i64.to_string());
    }
}

//! Customer domain types and repository boundary.

use ports::customer::CustomerRecord;

/// Domain view of a customer profile.
#[derive(Debug, Clone)]
pub struct CustomerProfile {
    pub record: CustomerRecord,
}

impl From<CustomerRecord> for CustomerProfile {
    fn from(record: CustomerRecord) -> Self {
        Self { record }
    }
}

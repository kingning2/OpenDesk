use crate::contracts::{AiAccount, AiProvider};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiIpcConfigResponse {
    pub providers: Vec<AiProvider>,
    pub accounts: Vec<AiAccount>,
}

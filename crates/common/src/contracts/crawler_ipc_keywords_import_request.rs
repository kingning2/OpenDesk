use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerIpcKeywordsImportRequest {
    pub trace_id: Option<String>,
    pub csv_content: String,
    pub batch_id: Option<String>,
}

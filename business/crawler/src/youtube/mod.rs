//! YouTube Data API 采集：任务编排、频道筛选与持久化。

mod api;
mod emit;
mod job;
mod keyword_generation;
mod service;

pub use emit::{CrawlerUIEmitter, CrawlerUIEvent, NoopCrawlerUIEmitter};
pub use keyword_generation::{
    build_keywords_user_prompt, parse_keyword_list, GenerateCrawlerKeywords, KEYWORDS_SYSTEM_PROMPT,
};
pub use service::CrawlerService;

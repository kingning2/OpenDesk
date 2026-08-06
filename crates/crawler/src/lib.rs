//! 桌面端进程内 YouTube 采集能力。

mod emit;
mod job;
mod keyword_generation;
mod service;
mod youtube;

pub use emit::{CrawlerUiEmitter, CrawlerUiEvent, NoopCrawlerUiEmitter};
pub use keyword_generation::{
    build_keywords_user_prompt, parse_keyword_list, GenerateCrawlerKeywords, KEYWORDS_SYSTEM_PROMPT,
};
pub use service::CrawlerService;

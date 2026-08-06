//! Agent AI 基建：模型协议、提示词结构与 Skill 注册。
//!
//! 本 crate **不放业务用例**。邮件、爬虫等领域逻辑留在各自 Feature crate，
//! 仅通过本 crate 提供的 `llm` / `prompt` / `skills` 基建组合调用。
//!
//! 约定：一个基建能力一个目录（`llm/`、`prompt/`、`skills/` …）。

pub mod embedding;
pub mod llm;
pub mod prompt;
pub mod skills;

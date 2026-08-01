# Crawler Domain

## 职责

从 YouTube 获取潜客频道、邮箱和来源元数据，并支持导入客户。

```text
React crawler → Tauri IPC → crates/crawler
                             ├─ YouTube API
                             ├─ crates/agent（关键词生成经 agent::llm）
                             └─ crawler.db
```

无邮箱频道可通过 `crates/crawler-enrich` 与 `opendesk-worker` 的 `crawler_email_enrich` handler 补全；结果保留并记录状态。

## 当前事实

- 任务启停、日志、频道结果与邮箱展示已实现。
- Rust 内置 YouTube 调用与 LLM 关键词生成。
- Worker 已包含邮箱补全 handler。
- 客户导入与产品验收状态以当前 Change/代码为准，不在本 Domain 复制实施清单。

## 边界

- MVP 获客渠道仅 YouTube。
- 导入客户按邮箱去重，来源元数据包含 channel id/url/title。
- Crawler 不自动发信，不维护客户商务状态。

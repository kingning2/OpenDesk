# Customer Domain

## 职责

客户档案、来源、联系方式、商务状态、报价、合作字段与沟通时间线。

```text
React customer → Tauri IPC → Rust Customer UseCase → SQLite
Agent/其他 Feature → 只读 Query Port
```

## 当前事实

- `crates/customer`、Storage 与桌面 Feature 已提供客户列表、详情、新建、编辑和邮箱唯一约束。
- `customer`、`quote_history`、`customer_timeline`、`cooperation_audit` 等表已存在。
- Crawler 导入、Mail/Channel 时间线和 Agent 只读上下文通过明确应用边界协作。

## 边界

- AI 只能读取或建议，不直接改客户、报价、合作状态。
- 邮件/渠道协议归各自领域。
- 客户写操作经 Rust UseCase；报价与合作变更保留审计。
- MVP 以邮箱作为全局去重键。

字段权威定义以 Contract 与 migration 为准，本 Domain 不复制完整表结构。

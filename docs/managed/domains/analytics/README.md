# Analytics Domain

## 状态

**planned（暂缓）** — 本领域原为 email-agent 商务**数据分析与报表**，依赖已移除的 **Mail / Customer / Channel** 数据。本期不实现。

## 职责（未来）

- 只读聚合查询（Rust SQL，不经 Python）
- 报表快照保存/删除
- 为 React 图表/表格提供 DTO

## 非职责

- 原始邮件/WA 消息存储（Mail / Channel 板块）
- KOL 频道统计（KOL 领域，暂缓）
- Python 直连 SQLite 聚合

## 当前状态

**未实现。** Email-Agent 移植项目已移除；依赖的 `mail_message` / `mail_open_event` / `customer` 等表已不存在，重新引入时按新 Change 设计。

## 当前约束

- 统计查询不得暴露凭据或完整正文给 Agent（除非单独只读工具评审）

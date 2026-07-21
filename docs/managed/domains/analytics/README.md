# Analytics Domain

## 职责

商务 **数据分析与报表**：漏斗、趋势、转化、开信追踪、WhatsApp 统计摘要。

- 只读聚合查询（Rust SQL，不经 Python）
- 报表快照保存/删除
- 开信事件统计（本地像素 `mail_open_event`）
- 为 React 图表/表格提供 DTO

## 非职责

- 原始邮件/WA 消息存储（Mail / Channel）
- KOL 频道统计（KOL 领域，暂缓）
- 实时 BI 平台、多租户报表
- Python 直连 SQLite 聚合

## 稳定边界

```text
React（数据分析 Tab）
  → Tauri IPC（analytics/*）
  → Rust 只读聚合查询
  → SQLite（mail_message / mail_open_event / channel_message / customer / report_snapshot）
```

## 入口

| 类型 | 路径 |
|------|------|
| Rust | `crates/analytics/`（待建） |
| Contract | `contracts/schema/v1/analytics/`（待建） |
| React | `apps/desktop/src/features/analytics/`（待建） |
| Epic | [EPIC-20260721-001](../../changes/2026/07/epic-20260721-001-email-agent-port.md) Phase 6 |

## 数据模型（规划）

| 表 | 用途 |
|----|------|
| `mail_open_event` | 开信像素命中 |
| `report_snapshot` | 用户保存的报表 JSON |

## 当前状态

**尚未实现。** email-agent 对应 `/api/stats*`、`/api/reports`。

## 当前约束

- 远程 Email Read API（kol-service）**暂缓**；本期仅本地开信像素
- 统计查询不得暴露凭据或完整邮件正文给 Agent（除非单独只读工具评审）

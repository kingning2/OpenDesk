# Schedule Domain

## 职责

**合作排期**管理：排期条目、关联客户、产品信息（本地字段）。

- 排期 CRUD
- 产品/推广条目维护（**不含** Gbyte 远程链接同步，暂缓）
- 与 Customer 关联展示

## 非职责

- KOL 合作库、Gbyte 渠道同步（KOL 领域）
- 邮件待发队列（Mail / `pending_send_queue`）
- 日历外部同步（Google Calendar 等）

## 稳定边界

```text
React（排期管理 Tab）
  → Tauri IPC（schedule/*）
  → Rust schedule UseCase
  → SQLite（schedule_entry / schedule_product）
```

## 入口

| 类型 | 路径 |
|------|------|
| Rust | `crates/` 实施时定（可并入 customer 或独立 schedule） |
| Contract | `contracts/schema/v1/schedule/`（待建） |
| React | `apps/desktop/src/features/schedule/`（待建） |
| Epic | [EPIC-20260721-001](../../changes/2026/07/epic-20260721-001-email-agent-port.md) Phase 6 |

## 数据模型（规划）

| 表 | 用途 |
|----|------|
| `schedule_entry` | 排期主记录 |
| `schedule_product` | 产品/推广条目（本地 URL 字段可空） |

## 当前状态

**尚未实现。** email-agent 对应 `schedule-data.json`。

## 当前约束

- Gbyte 推广链接字段预留可空列，对接归入 Phase 7 KOL Epic

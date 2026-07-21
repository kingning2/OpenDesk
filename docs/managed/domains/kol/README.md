# KOL Domain

## 状态

**planned** — 本期 **不实现、不对接** Gbyte/KOL 平台；仅预留扩展点，待独立 KOL 系统迁入后启动 Phase 7 Epic。

## 职责（未来）

- KOL 频道库 CRUD、导入导出
- 关键词批次、YouTube 爬取任务
- 合作记录（collaborations）
- KOL Helper 自动调度
- 外部服务适配（Gbyte 等）

## 非职责

- YouTube **获客爬虫**（Crawler 领域 — 抽邮箱导入 Customer，已独立推进）
- 邮件/WhatsApp 主链路
- 本期任何 Gbyte HTTP 调用

## 稳定边界（规划）

```text
React（KOL 频道 Tab — 当前 Coming Soon）
  → Tauri IPC（kol/*）  [未实现]
  → Rust KolChannelPort / KolSyncPort  [trait 桩]
  → 外部 KOL 服务或本地 kol_* 表  [未来]
```

## 预留入口

| 类型 | 路径 |
|------|------|
| Port trait | `crates/ports/src/kol.rs`（桩，Phase 0） |
| Contract 占位 | `contracts/schema/v1/kol/_reserved/README.md` |
| React | `apps/desktop/src/features/kol/` 或导航占位页 |
| Adapter | `crates/adapter/gbyte/`（空壳，未来） |

## 数据模型（未来 DDL 草案，本期不 migration）

| 表 | 用途 |
|----|------|
| `kol_channel` | 频道主数据 |
| `kol_keyword` | 关键词 |
| `kol_collaboration` | 合作记录 |

## 与 Customer 关系

- 获客阶段：`customer.source_meta` 可存 YouTube 频道 JSON（Crawler 写入）
- KOL 迁入后：可选 `kol_channel_id` 外键或同步任务，实施时定

## 当前约束

- **禁止** 在本期 Change 中实现 Gbyte API、kol.db 迁移、远程反向代理
- Phase 7 启动前须评审本 README 并新建独立 Epic

## 相关

- [EPIC-20260721-001](../../changes/2026/07/epic-20260721-001-email-agent-port.md) — Phase 7
- [Crawler 领域](../crawler/README.md) — 获客，非 KOL 平台

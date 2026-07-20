# Storage Domain

## 职责

OpenDesk 本地持久化的 **唯一 Rust 所有权**：

- SQLite 文件路径与 Migration（Diesel）
- `crawler.db` 与 `opendesk.db` 双库策略
- Repository / Diesel models 供各 Feature crate 通过 Port 访问
- WAL 与 `PRAGMA foreign_keys` 启动配置
- Worker 与主进程 **共享 `opendesk.db`** 的并发约定

## 非职责

- 业务 UseCase 规则（各 Feature crate）
- Python / React 直连 SQLite
- OCR 引擎实现（Worker + OCR 领域）
- 密钥明文存储（mail `password_ref` 指向 OS secure store）

## 稳定边界

```text
Feature crates (customer, mail, ocr, …)
  → ports::*Store traits
  → storage/src/*_db/ (Infrastructure)
  → SQLite 文件

Tauri 主进程 app
  → 轻量 SQL + job 入队

opendesk-worker
  → background_job 消费 + OCR 结果写入
```

## 入口

| 类型 | 路径 |
|------|------|
| Crate | `crates/storage/` |
| Crawler DB | `crates/storage/src/crawler_db/`, `migrations/*crawler*` |
| 业务 DB（规划） | `crates/storage/src/opendesk_db/`（待建） |
| Schema 设计 | [`docs/architecture/database-schema.md`](../../../architecture/database-schema.md) |
| 进程模型 | [`docs/architecture/process-model.md`](../../../architecture/process-model.md) |
| ADR | [ADR-0002 Worker 进程](../../decisions/runtime/adr-0002-heavy-work-worker-process.md) |

## 库文件

| 文件 | 表域 |
|------|------|
| `{data}/OpenDesk/crawler.db` | 爬虫（**已有**） |
| `{data}/OpenDesk/opendesk.db` | CRM、邮件、价目、渠道、OCR、后台任务（**规划**） |

完整表清单见 [`database-schema.md`](../../../architecture/database-schema.md)。

## 当前状态

- `crawler.db` Migration 与 Diesel schema **已实现**
- `opendesk.db` **未实现**；表设计已在 architecture 文档定稿

## 当前约束

- Migration 只增不破坏；跨库逻辑外键（crawler → customer）在应用层校验
- Worker 写 OCR 时主进程可读；须 WAL
- 所有新表 Change 须更新 `database-schema.md` 或链接至新 Migration Change

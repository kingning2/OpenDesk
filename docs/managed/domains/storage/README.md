# Storage Domain

## 职责

DingDa 本地持久化的 **唯一 Rust 所有权**：

- 通用 SQLite `RecordStore` 实现（`ports::repository`），供各使用方经 Port 访问
- 业务表与领域专用库（crawler.db / dingda.db）**已随采集/客户/邮箱板块移除**，如后续需要按新 Change 重新引入

## 非职责

- 业务 UseCase 规则（业务代码在 `apps/desktop/src-tauri`）
- Python / React 直连 SQLite
- 领域专属表 DDL（业务重构回时再规划）

## 稳定边界

```text
UseCase（src-tauri / future crates）
  → ports::repository::RecordStore trait
  → storage/src/repository/（SQLite Infrastructure）
  → SQLite 文件
```

## 入口

| 类型 | 路径 |
|------|------|
| Crate | `crates/storage/` |
| 通用存储实现 | `crates/storage/src/repository/` |
| Port trait | `crates/ports/src/repository.rs` |

## 当前状态

`crates/storage` 提供通用 `RecordStore`（SQLite）骨架，无任何领域业务表。领域专用数据模型为未来规划，未实现。

## 当前约束

- SQLite 仅经 Rust 访问；Python / React 禁止直连
- 业务表重构须先建 Change Record

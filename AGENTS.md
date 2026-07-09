# OpenDesk 分支说明（role/python）

本分支面向 **Developer C（Python/AI Runtime）**。

## 分支

- `role/frontend`：Developer A（React/TS/UI）
- `role/rust`：Developer B（Rust/Tauri/SQLite/IPC）
- `role/python`：Developer C（Python/AI Runtime）

## 规范入口

- `.cursor/rules/`：各角色规则文件
- `.cursor/rules/python.md`：本分支规则
- `.cursor/rules/master.md`：全仓库基线规则（所有分支都必须遵守）
- `contracts/`：三端共享契约（DTO/IPC/HTTP/Event/Error），任何 Breaking Change 必须先改契约


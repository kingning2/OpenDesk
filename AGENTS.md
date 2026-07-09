# OpenDesk 分支说明（默认）

本仓库采用三人并行开发模式，每位开发者在各自分支维护自己的 Cursor rules 与代码规范。

## 分支

- `role/frontend`：Developer A（React/TS/UI）
- `role/rust`：Developer B（Rust/Tauri/SQLite/IPC）
- `role/python`：Developer C（Python/AI Runtime）

## 规范入口

- `.cursor/rules/`：各角色规则文件
- `contracts/`：三端共享契约（DTO/IPC/HTTP/Event/Error），任何 Breaking Change 必须先改契约


本目录存放 Cursor Rules（团队代码规范与协作边界）。

- `frontend.md`：React/桌面端规范（Developer A）
- `rust.md`：Rust/Tauri/SQLite/IPC 规范（Developer B）
- `python.md`：Python/AI Runtime 规范（Developer C）

规则目标：
- 三人并行开发时减少冲突
- 保持架构边界：React → Rust → Python
- 避免 Breaking Change（契约优先）


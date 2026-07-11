# Cursor Rules

本目录存放 Cursor Rules（团队代码规范与协作边界）。

## 规则文件

| 文件 | 范围 | 说明 |
|------|------|------|
| [`master.md`](master.md) | 全仓库 | **基线规则，所有分支必须遵守** |
| [`frontend.md`](frontend.md) | 前端 | React Compiler · @desk/ui · 设计系统 |
| [`rust.md`](rust.md) | Rust / Tauri | crates · IPC · 六边形分层 |
| [`../../skills/opendesk/`](../../skills/opendesk/) | 全仓库 | AI 开发知识库 |

> `python.md` 待 `role/python` 分支补充；前端细则见 `frontend.md` 与 `skills/opendesk/guides/ui-design-system.md`。

## 规则目标

- 三人并行开发时减少冲突
- 保持架构边界：React → Rust → Python
- 契约优先，避免 Breaking Change
- 当前阶段禁止业务逻辑与 Demo

## 快速约束

| 禁止 | 必须 |
|------|------|
| React → Python / SQLite | 跨端先改 `contracts/` |
| Python → SQLite / React | Feature 间走 Event 或 Query Port |
| Feature 间直接 import | UseCase 只依赖 Ports trait |
| UseCase 中写 IO | `pnpm lint` 通过后再提交 |
| 业务逻辑 / Demo（当前阶段） | 最小修改，不臆测需求 |

完整内容见 [`master.md`](master.md)。

# OpenDesk

企业级 AI 智能客服桌面平台（Architecture v2 + Naming v1）。

> **当前阶段：Architecture Skeleton** — 只允许骨架、契约、规范与工具链；禁止业务逻辑与 Demo。

## 架构约束（必读）

```
React  →  Rust  →  Python
         ↑
    唯一协调者
```

| 禁止 | 说明 |
|------|------|
| React → Python | 含 localhost HTTP / WebSocket / SSE |
| React → SQLite | 存储由 Rust 负责 |
| Python → SQLite | 存储由 Rust 负责 |
| Feature 间直接 import | 跨 Feature 只允许 Query Port · Event · Contract |
| UseCase 中写 IO | SQL / HTTP / 文件系统须在 Infrastructure 层 |
| 先改实现再改契约 | 跨端变更须先改 `contracts/` |

完整规则见 [`.cursor/rules/master.md`](.cursor/rules/master.md)。

## 结构

- `apps/desktop` — Tauri + React 桌面应用
- `packages` — 前端共享包（ui · platform · contracts）
- `crates` — Rust Workspace（kernel · app · feature crates）
- `python` — AI Runtime（sidecar · gateway · queue · worker · provider）
- `contracts` — 三端共享契约（**唯一真相源**）

## 开发

```bash
pnpm install
pnpm tauri dev
```

## 代码校验

```bash
pnpm lint              # 三端全量检查（含 TypeScript 类型检查）
pnpm lint:frontend     # ESLint + tsc
pnpm lint:types        # TypeScript 类型检查
pnpm lint:rust         # rustfmt + clippy
pnpm lint:python       # ruff check + format
pnpm lint:fix          # 自动修复（前端 + rust fmt + python）
```

提交前 Husky 会自动对 staged 文件跑对应语言的 lint（`pnpm install` 后生效）。

## 文档

- [`skills/opendesk/`](skills/opendesk/) — **AI 开发知识库**（架构 · 规范 · 模板 · 脚本）
- [`.cursor/skills/opendesk/SKILL.md`](.cursor/skills/opendesk/SKILL.md) — Cursor Skill 入口
- [`.cursor/rules/master.md`](.cursor/rules/master.md) — 全局架构约束与协作边界
- [`AGENTS.md`](AGENTS.md) — 分支与角色说明
- [`contracts/README.md`](contracts/README.md) — 契约层说明
- `docs/architecture/` — 架构与 ADR

### 架构检查

```bash
python skills/opendesk/scripts/check_architecture.py
```

# OpenDesk

本地优先 **AI 商务桌面**：YouTube 获客 → 邮件谈价 → WhatsApp 辅助（翻译/建议，人发）。

> **当前阶段：** Architecture Skeleton + 部分垂直切片（爬虫、UI Shell、Sidecar）。MVP 商务能力见下方规划文档。

## MVP 规划（团队评审）

👉 **[`docs/managed/MVP_REVIEW.md`](docs/managed/MVP_REVIEW.md)** — GitHub 评审入口：路线图、Epic、子任务改动范围、ADR。

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
| Python → SQLite | 存储由 Rust 负责；AI 仅只读 Query Port |
| AI 写库 / 自动发信 | 客户状态与发送仅 UI 人工操作 |
| Feature 间直接 import | 跨 Feature 只允许 Query Port · Event · Contract |
| 先改实现再改契约 | 跨端变更须先改 `contracts/` |

完整规则见 [`.cursor/rules/master.md`](.cursor/rules/master.md)。

## 结构

- `apps/desktop` — Tauri + React 桌面应用
- `packages` — 前端共享包（ui · platform · contracts）
- `crates` — Rust Workspace（kernel · app · feature crates）
- `python` — AI Runtime（sidecar · gateway · queue · worker · provider）
- `contracts` — 三端共享契约（**唯一真相源**）
- `docs/managed/` — MVP 规划、Change Record、ADR

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

- **[`docs/managed/MVP_REVIEW.md`](docs/managed/MVP_REVIEW.md)** — MVP 团队评审入口
- [`docs/managed/roadmaps/mvp-sales-workbench.md`](docs/managed/roadmaps/mvp-sales-workbench.md) — 总路线图
- [`skills/opendesk/`](skills/opendesk/) — AI 开发知识库
- [`.cursor/skills/opendesk/SKILL.md`](.cursor/skills/opendesk/SKILL.md) — Cursor Skill 入口
- [`.cursor/rules/master.md`](.cursor/rules/master.md) — 全局架构约束
- [`docs/architecture/`](docs/architecture/) — 产品与运行时架构
  - [`product-architecture.md`](docs/architecture/product-architecture.md) — 商务工作台产品架构
  - [`python-ai-runtime-architecture.md`](docs/architecture/python-ai-runtime-architecture.md) — Python AI Runtime

### 架构检查

```bash
python skills/opendesk/scripts/check_architecture.py
```

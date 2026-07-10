# Architecture

OpenDesk 架构文档与 ADR 存放目录。

## 三层边界

```
┌─────────────────────────────────────────┐
│  React（展示层）                          │
│  UI · State · Theme · Animation         │
│  禁止：业务逻辑 · AI · SQL · 直连 Python  │
└──────────────────┬──────────────────────┘
                   │ Tauri IPC（@desk/platform/ipc）
┌──────────────────▼──────────────────────┐
│  Rust Application Core（唯一协调者）      │
│  Storage · Event Bus · Task · Sidecar   │
│  禁止：unwrap · Feature 直调 · 阻塞 UI    │
└──────────────────┬──────────────────────┘
                   │ 本机 IPC（contracts 定义）
┌──────────────────▼──────────────────────┐
│  Python AI Runtime                      │
│  LLM · RAG · OCR · Agent · Queue        │
│  禁止：GUI · SQLite · 业务状态 · React    │
└─────────────────────────────────────────┘
```

## 设计原则

1. Contracts First
2. Feature First
3. Dependency Inward
4. Event Driven
5. Local First / Offline First
6. Testable by Design
7. Composition over Inheritance
8. Explicit over Implicit

## Feature 列表（互相独立）

`chat` · `mail` · `agent` · `workflow` · `knowledge` · `browser` · `ocr` · `mcp` · `plugin` · `tenant` · `user` · `channel`

跨 Feature 通信只允许：

- **Query Port** — 只读查询
- **Event** — Pub/Sub（`kernel::event`）
- **Contract** — 共享 DTO

## 六边形架构

```
UseCase  →  Ports (trait)  →  Infrastructure
```

UseCase 层禁止直接接触：SQL · HTTP · Filesystem · SQLite · Tauri · Python

## 契约变更流程

```
contracts/  →  codegen  →  Rust  →  Python  →  React
```

禁止跳过 `contracts/` 直接改实现。

## 相关文档

- [`.cursor/rules/master.md`](../../.cursor/rules/master.md) — 完整约束与 lint 规范
- [`contracts/README.md`](../../contracts/README.md) — 契约层说明
- [`contracts/compatibility/MIGRATION.md`](../../contracts/compatibility/MIGRATION.md) — 破坏性变更迁移
- `adr/` — Architecture Decision Records

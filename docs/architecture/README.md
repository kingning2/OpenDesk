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
│  Agent · License · Sidecar · Event Bus  │
│  禁止：unwrap · Feature 直调 · 阻塞 UI    │
└──────────────────┬──────────────────────┘
                   │ 本机 IPC（contracts 定义）
┌──────────────────▼──────────────────────┐
│  Python Sidecar                         │
│  LLM · Agent（只读 Query）               │
│  禁止：GUI · SQLite · 写库 · 自动发信      │
└─────────────────────────────────────────┘
```

## 代码布局

- `crates/` — 只放**基建代码**（与业务无关的基础设施）：`adapter` · `common` · `kernel` · `ports` · `runtime` · `storage`
- `apps/desktop/src-tauri/` — 业务代码与 Tauri 组装：`agent` · `license` · `commands` · `state` · `platform` · `logging`
- `apps/desktop/src/` — React 前端（`features/` + `route/` + `i18n/`）
- `python/` — Python sidecar 与契约包
- `contracts/` — 跨端契约生成源

业务代码直接放在 `src-tauri`，不放入 `crates/`。

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

`chat` · `agent` · `knowledge`

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

| 文档 | 说明 |
|------|------|
| [`whatsapp-webhook-deployment.md`](whatsapp-webhook-deployment.md) | **WA webhook** 部署手册（**历史参考**，已被 ADR-0006 Baileys 方案取代） |
| [`.cursor/rules/master.md`](../../.cursor/rules/master.md) | 完整约束与 lint 规范 |
| [`contracts/README.md`](../../contracts/README.md) | 契约层说明 |

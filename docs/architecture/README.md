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
│  LLM · Agent（只读 Query）· 邮件/WA 草稿   │
│  禁止：GUI · SQLite · 写库 · 自动发信      │
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

`chat` · `mail` · `agent` · `workflow` · `knowledge` · `browser` · `ocr` · `mcp` · `plugin` · `tenant` · `user` · `channel` · `crawler`

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
| [`database-schema.md`](database-schema.md) | **SQLite 双库**、全表 DDL、ER 图 |
| [`process-model.md`](process-model.md) | **三进程模型**（UI / Worker / Sidecar）；OCR 不得阻塞 UI |
| [`product-architecture.md`](product-architecture.md) | **商务工作台**产品架构（获客·邮件·WA 辅助） |
| [`whatsapp-webhook-deployment.md`](whatsapp-webhook-deployment.md) | **WA webhook** 开发隧道与部署手册（MVP M5） |
| [`python-ai-runtime-architecture.md`](python-ai-runtime-architecture.md) | Python AI Runtime 工程架构 |
| [`../managed/MVP_REVIEW.md`](../managed/MVP_REVIEW.md) | **MVP 团队评审入口**（路线图与子任务） |
| [`.cursor/rules/master.md`](../../.cursor/rules/master.md) | 完整约束与 lint 规范 |
| [`contracts/README.md`](../../contracts/README.md) | 契约层说明 |

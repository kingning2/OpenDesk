# OpenDesk

企业级 AI 智能客服桌面平台（Architecture v2 + Naming v1）。

## 结构

- `apps/desktop` — Tauri + React 桌面应用
- `packages` — 前端共享包（ui · platform · contracts）
- `crates` — Rust Workspace（kernel · app · feature crates）
- `python` — AI Runtime（sidecar · gateway · queue · worker · provider）
- `contracts` — 三端共享契约

## 开发

```bash
pnpm install
pnpm tauri dev
```

## 文档

- `docs/architecture/` — 架构与 ADR

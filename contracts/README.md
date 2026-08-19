# DingDa Contracts

三端共享契约层 — **唯一真相源**（DTO · IPC · HTTP · Event · Error）。

## 变更流程（禁止跳过）

```
1. 修改 Contract（本目录）
       ↓
2. Code Generation（codegen/）
       ↓
3. 受影响实现端：默认 Rust → React
   仅当该能力必须走 sidecar 时才改 Python
```

**禁止**先改实现再补契约。临时原型须在 PR 标注并尽快补全。

## 目录

- `schema/v1/` — JSON Schema
- `openapi/` — OpenAPI 规范（含 sidecar 管理面 `sidecar.v1.yaml` 与 `sidecar.paths/`）
- `codegen/` — 生成 TypeScript / Rust / Python
- `compatibility/` — 字段规则与迁移指南

## Breaking Change

1. 新增 `schema/v2`（或新文件）
2. 更新 [`compatibility/MIGRATION.md`](compatibility/MIGRATION.md)
3. 更新 [`CHANGELOG.md`](CHANGELOG.md)
4. PR 至少 2 人 Approve

## 相关文档

- [`.cursor/rules/master.md`](../.cursor/rules/master.md) — 全局架构约束
- [`compatibility/FIELD_RULES.md`](compatibility/FIELD_RULES.md) — 字段变更规则

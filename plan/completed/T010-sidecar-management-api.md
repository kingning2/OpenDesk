| Field | Value |
|-------|-------|
| ID | T010 |
| Priority | P1 |
| Status | completed |
| Depends on | T004 |
| Blocks | — |
| Milestone | M2 |

## Goal

Rust 侧实现 sidecar 管理面 HTTP 客户端（`/health` 已有，补 `/stats` `/tasks/active` `/metrics` `/debug/dump`）。

## Scope

- 契约对齐 `contracts/openapi/sidecar.v1.yaml`
- `SidecarClient::get_json` 路由绑定模块
- Python sidecar 管理面 handler 骨架（返回空对象 / 占位）

## Out of scope

- Prometheus 真实指标
- React 暴露管理面

## Acceptance criteria

- [x] Rust client 可调用至少 3 个管理面 endpoint
- [x] Python sidecar 对应 route 返回 200 JSON
- [x] 仅 Rust 可调用（架构 boundary 不变）
- [x] `pnpm lint` 通过

## Key files

- `contracts/openapi/sidecar.v1.yaml`
- `crates/runtime/src/sidecar/client.rs`
- `crates/runtime/src/sidecar/routes/`
- `python/sidecar/sidecar/server.py`

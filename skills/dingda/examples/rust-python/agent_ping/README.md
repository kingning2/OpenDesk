# Rust ↔ Python: `/v1/agent/ping`

探活骨架，不是「AI 必须在 Python」的依据。新 Agent / LLM 能力默认在 Rust 实现（ADR-0009）。

| 层 | 文件 |
|----|------|
| Contract | `contracts/schema/v1/agent/sidecar/ping.*.schema.json` |
| OpenAPI | `contracts/openapi/sidecar.paths/agent_ping.yaml` |
| Rust client | `crates/runtime/src/sidecar/routes/agent_ping.rs` |
| Python handler | `python/packages/gateway/.../handlers/agent_ping.py` |

React **禁止**直连此路径。

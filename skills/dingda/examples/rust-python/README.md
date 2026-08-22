# Rust ↔ Python 通讯示例

仅在 ADR-0009 例外（Rust 生态不够）时走这条路径。默认 AI / 业务在 Rust 完成，不经过 Python。

DingDa 中 **Rust → Python** 走本机 HTTP sidecar；**Python → React** 必须经 Rust 转发。

## 端到端路径

```
React
  │  Tauri IPC（@desk/platform/ipc）
  ▼
apps/desktop/src-tauri/commands
  │
  ▼
apps/desktop/src-tauri/src/<feature>.rs（UseCase）
  │
  ▼
crates/infra/src/sidecar（HTTP client）
  │  POST http://127.0.0.1:{port}/v1/{feature}/{action}
  ▼
python/sidecar（HTTP server，Rust 管理生命周期）
  │
  ▼
python/packages/gateway/handlers/{feature}_{action}.py
```

## 内置示例：`agent_ping`

运行脚手架生成完整骨架：

```bash
python skills/dingda/scripts/create_rust_python_ipc.py --feature agent --action ping
```

生成后阅读：

- [agent_ping/README.md](agent_ping/README.md)
- [../rust/sidecar_client.rs](../rust/sidecar_client.rs)
- [../python/sidecar_handler.py](../python/sidecar_handler.py) — handler 模式

## 流式示例

Agent / LLM token 流（仅 sidecar 例外）：

```
Python yield token
  → Rust read HTTP stream
  → app.emit("agent:token", payload)
  → React platform.listen
```

见 `create_rust_python_ipc.py --streaming` 生成的 `stream.rs` 骨架。

## 管理面（无需按 feature 生成）

已定义于 `contracts/openapi/sidecar.v1.yaml`：

- `GET /health`
- `GET /stats`
- `GET /tasks/active`
- `GET /metrics`
- `GET /debug/dump`

## 禁止

- React fetch localhost
- Python import tauri / 调前端

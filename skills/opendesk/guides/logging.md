# Logging Guide

## Rust（tracing）

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(ctx), fields(trace_id = %trace_id, task_id = %task_id))]
pub async fn run_agent(ctx: &AppCtx, trace_id: &str, task_id: &str) -> Result<()> {
    info!("agent run started");
    // ...
    Ok(())
}
```

### 字段规范

| 字段 | 用途 |
|------|------|
| `trace_id` | 端到端请求链 |
| `task_id` | 异步任务 |
| `feature` | chat / mail / agent |
| `tenant_id` | 多租户（若适用） |

## Python

```python
import logging

logger = logging.getLogger("opendesk.gateway")

def handle_request(trace_id: str) -> None:
    logger.info("request received", extra={"trace_id": trace_id})
```

Rust **必须接管** sidecar stdout/stderr 并写入结构化日志。

## 前端

- 开发环境：`console.debug` 仅用于 UI 调试
- 生产：错误上报经 IPC 到 Rust，不在前端持久化日志文件

## 禁止

- 日志中输出密钥、token、PII 明文
- `println!` / `print()` 作为生产日志手段

## 相关

- [error.md](error.md)
- [../architecture/layers.md](../architecture/layers.md)

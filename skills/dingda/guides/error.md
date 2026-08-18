# Error Guide

## 分层错误策略

| 层 | 方式 |
|----|------|
| Rust domain | `thiserror` 枚举 |
| Rust IPC | 映射为 `ErrorDto`（contracts） |
| Python | 自定义异常 → Rust 可解析的错误码 |
| React | 展示 `ErrorDto.message`，不解析内部栈 |

## Contract 错误定义

```
contracts/schema/v1/common/error/base.schema.json
contracts/schema/v1/<feature>/error/<code>.schema.json
```

```json
{
  "type": "object",
  "required": ["code", "message"],
  "properties": {
    "code": { "type": "string" },
    "message": { "type": "string" },
    "trace_id": { "type": "string" }
  }
}
```

## Rust 示例

```rust
#[derive(Debug, Error)]
pub enum PortError {
    #[error("not found")]
    NotFound,
    #[error("storage: {0}")]
    Storage(String),
}
```

## 禁止

- `unwrap()` / `expect()` 于生产路径
- 吞掉错误（空 `catch`）
- 向用户暴露内部 Rust/Python 栈（除非 debug 模式经 IPC 开关）

## 相关

- [rust.md](rust.md)
- [contracts.md](contracts.md)

# Repository Template

Storage 层 Port 实现骨架。

## 结构

```
crates/ports/src/{{NAME}}_repo.rs    # trait
crates/storage/src/{{NAME}}_repo.rs  # SQLite impl (skeleton)
```

## trait 骨架

```rust
pub trait {{TRAIT}}: Send + Sync {
    // TODO: CRUD signatures — no SQL here in trait
}
```

## 禁止

- UseCase 内 SQL
- Python 访问 SQLite

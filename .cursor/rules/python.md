# Python Rules（Developer C）

适用范围：`python/**`

## 职责边界

- Python 是 **AI Runtime / Sidecar**，只被 Rust `runtime` 调用
- **禁止**直连 SQLite、React、Tauri
- **禁止** import `apps/`、`crates/`、`packages/` 源码

## 目录

```
python/
├── sidecar/           # HTTP 服务（Rust 管理生命周期）
└── packages/          # 共享 Python 包
```

## 契约

- 请求/响应 DTO 来自 `contracts/` codegen（`python/packages/contracts`）
- Breaking Change：先改 `contracts/schema`，再 `sync_contracts.py`

## 日志

- 结构化 JSON 日志；`trace_id` 由 Rust 传入或从 header 解析
- 禁止 `print()` 调试残留

## 相关

- `skills/opendesk/guides/python.md`
- `skills/opendesk/recipes/add-python-package.md`

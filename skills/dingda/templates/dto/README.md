# DTO Template

跨端数据传输对象，定义于 contracts，经 codegen 同步到受影响端（默认 Rust / TypeScript；仅 sidecar 例外才同步 Python）。

## TODO

- [ ] 与 IPC/Event schema 复用或引用（$ref）
- [ ] 不含业务逻辑

## 示例路径

`contracts/schema/v1/<feature>/dto/<name>.schema.json`

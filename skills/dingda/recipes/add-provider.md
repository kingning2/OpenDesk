# Recipe: Add Provider

外部 API 适配器**默认用 Rust**（`src-tauri` 或 adapter crate）。

仅当该 provider 的官方/成熟库只存在于 Python，且 Change Record 论证了生态缺口时，才放到 `python/packages/`。

## 修改顺序（默认 Rust）

1. Contract: 配置 DTO（无密钥字段明文 schema）
2. Rust adapter / UseCase 调用 HTTP API
3. 密钥只经 runtime 配置注入，不写入契约示例

## 修改顺序（仅 ADR-0009 例外）

1. `python/packages/<name>/` 骨架
2. Contract: 配置 DTO
3. Rust 经 sidecar 传递配置

## 禁止

- API Key 写入契约示例或日志
- Provider 内持久化
- 未论证就把 provider 放 Python

## 模板

[../templates/provider/](../templates/provider/)

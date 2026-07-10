# Recipe: Add Provider

## 修改顺序

1. `python/packages/provider/src/provider/<name>.py` 骨架
2. Contract: 配置 DTO（无密钥字段明文 schema）
3. Rust 经 runtime 传递配置给 Python

## 禁止

- API Key 写入契约示例或日志
- Provider 内持久化

## 模板

[../templates/provider/](../templates/provider/)

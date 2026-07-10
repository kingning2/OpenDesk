# Recipe: Add Plugin

## 修改顺序

1. Contract: `contracts/schema/v1/plugin/`
2. Crate: `crates/plugin/` 扩展（或子模块）
3. Feature 注册点骨架
4. 前端 `features/plugin/` 市场页占位

## 禁止

- 插件绕过 IPC 直连 Rust 内部
- 插件内嵌 Python

## 模板

[../templates/plugin/](../templates/plugin/)

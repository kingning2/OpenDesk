# Release Guide

发布前：

```bash
pnpm lint
pnpm check:architecture
pnpm contracts:check
pnpm tauri:build:locked
```

- 契约变化已追加 CHANGELOG；Breaking Change 有迁移说明。
- Tauri 安装包只包含前端资源、Rust 二进制与明确配置的外部资源。
- API Key 和凭据不进入仓库、日志或构建产物。
- `frontend/<kind>/<slug>` 与 `contract/<kind>/<slug>` 经 PR 合入；切换分支运行 `pnpm branch:sync`。

版本号与发布说明应描述用户可见变化，不复述实现文件列表。

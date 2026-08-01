---
name: verification-loop
description: OpenDesk 最小验证闭环：lint、React/Rust 架构检查与 Contract 生成物检查。
---

# Verification Loop

完成可提交改动、跨层修改或 PR 前运行：

```bash
pnpm lint
pnpm check:architecture
pnpm contracts:check
```

任一步失败则停止提交，先修复架构或 Contract 不一致，再处理风格问题。不得通过修改 lint/check 配置绕过失败。

# Code Review Guide

按顺序检查：

1. 变更是否在当前分支 scope 与 Change Record 范围内。
2. 跨端字段是否先改 Contract，生成物是否同步。
3. React 是否只经 `@desk/platform/ipc` 调 Rust。
4. Feature 间是否通过 Event、Query Port 或 Contract。
5. Rust 用例是否把 SQL、HTTP、文件与框架细节留在 Infrastructure。
6. 错误、日志、凭据与后台任务边界是否安全。
7. 测试是否覆盖真实风险，注释是否只解释契约或原因。

```bash
pnpm lint
pnpm check:architecture
pnpm contracts:check
```

P0：架构绕过、数据破坏、密钥泄漏；P1：功能错误或明显回归；P2：可维护性与缺失测试。

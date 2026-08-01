# Testing Guide

- Domain 规则用纯 Rust 单元测试。
- Application 用 Mock Port 验证编排、错误与状态转换。
- Infrastructure 测试真实序列化、SQLite migration 或协议边界。
- Tauri/React 测试以用户可见行为和 IPC Contract 为边界。
- Workflow、Worker、IMAP 等长任务至少覆盖恢复、重复执行或取消中的实际风险。

不为简单 getter 或类型定义堆测试。修复缺陷时优先增加能复现根因的最小测试。

```bash
cargo test --workspace
pnpm lint
pnpm check:architecture
pnpm contracts:check
```

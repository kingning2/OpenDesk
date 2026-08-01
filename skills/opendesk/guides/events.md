# Events Guide

使用 Event：

- 跨 Feature 写操作后的状态传播；
- 后台任务、Workflow 或 Worker 状态通知；
- Rust 向 React 推送异步进度。

需要同步返回值时使用同 Feature 调用或 Query Port，不用 Event 模拟 RPC。

步骤：

1. 在 Contract 定义 payload。
2. `pnpm contracts:sync`。
3. Rust Publisher 发布稳定 topic。
4. Subscriber 幂等处理。
5. 给 React 的通知由 Tauri Event 单独映射。

禁止魔法 topic、在 payload 携带密钥、依赖订阅顺序。

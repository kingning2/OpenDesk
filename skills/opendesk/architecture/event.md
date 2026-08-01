# Event

Rust 进程内跨 Feature 状态传播使用 `kernel::event`；Rust 向 React 通知使用 Tauri Event，两者不要混用。

## 规则

1. Event payload 先在 `contracts/schema/v1/<feature>/event/` 定义。
2. Publisher 只知道 topic 和 payload，不依赖 Subscriber。
3. Subscriber 必须考虑重复投递与乱序边界。
4. 需要立即返回数据时使用 Query Port 或同 Feature 的直接调用，不用 Event 模拟 RPC。

命名使用 `<feature>.<entity>.<past-tense-verb>`，例如 `mail.message.synced`。

```text
Feature A → kernel::event → Feature B
Rust      → Tauri Event   → React listener
```

新增后运行 `pnpm contracts:sync`、`pnpm contracts:check` 和相关 Rust 测试。

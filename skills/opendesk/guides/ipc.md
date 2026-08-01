# IPC Guide

调用链固定为：

```text
React Feature → @desk/platform/ipc → Tauri command → crates/app → Rust UseCase
```

1. 在 `contracts/schema/v1/<feature>/ipc/` 定义 request/response。
2. 运行 `pnpm contracts:sync`。
3. 在 Rust 应用层实现命令，并由 Tauri 壳注册。
4. 在 `packages/platform` 封装调用，Feature hook 使用封装。
5. 长任务返回 task/run id，通过 Tauri Event 推送状态；不要在 React 轮询内部实现。

错误必须映射为稳定 Contract；Feature 禁止直接 `invoke()`。

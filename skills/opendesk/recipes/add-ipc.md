# Recipe: Add IPC

1. 在 Contract 定义 request/response/error。
2. `pnpm contracts:sync`。
3. 在 `crates/app` 实现薄 command 并由 Tauri 注册。
4. 在 `packages/platform` 封装命令名与序列化。
5. Feature hook 调封装，不直接 `invoke()`。

长任务返回 id 并使用 Tauri Event；验证无效输入、Rust 错误映射和生成类型。模板见 [`../templates/ipc/`](../templates/ipc/)。

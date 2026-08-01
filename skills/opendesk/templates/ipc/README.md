# IPC Template

1. 在 Contract 定义 request/response/error。
2. 运行 `pnpm contracts:sync`。
3. Rust command 调用 Application 用例。
4. `packages/platform` 封装 IPC，Feature hook 使用封装。

Feature 禁止直接 `invoke()`。TypeScript 适配骨架见 [`platform.ts.tpl`](platform.ts.tpl)。

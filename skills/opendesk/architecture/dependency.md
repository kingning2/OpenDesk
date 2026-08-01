# 依赖规则

## 主调用链

```text
React Feature → @desk/platform → Tauri → crates/app → Feature/Application
                                                    → Ports ← Infrastructure
                                                    → crates/agent
```

## Rust

- `kernel`、`common`、`ports` 不依赖 Feature crate。
- Feature crate 不依赖其他 Feature 的内部实现。
- Infrastructure 实现 Port；Application 不反向依赖具体 IO。
- `crates/app` 和 Tauri 壳可以装配所有实现。
- `crates/agent` 提供模型与 Prompt 基础设施，领域 Prompt 内容和业务规则留在调用 Feature。

## React

- `packages/ui` 不依赖 IPC 或业务。
- `packages/platform` 不承载 Feature 规则。
- Feature 可依赖 `ui`、`platform`、`contracts`，不可导入其他 Feature 内部文件。

运行 `pnpm check:architecture` 检查关键边界。

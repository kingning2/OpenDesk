# Naming Guide

- 目录、crate、模块优先短名词：`mail`、`crawler`、`llm`、`storage`。
- 避免无信息后缀：`manager`、`helper`、`util`、`system`、`processor`。
- Rust 文件/模块用 `snake_case`，类型用 `PascalCase`，函数用 `snake_case`。
- TypeScript 文件用 kebab-case，组件/类型用 `PascalCase`，hook 用 `useXxx`。
- IPC 与 Event 名称保持稳定、带领域前缀；不要把实现技术写进业务命名。

运行 `pnpm check:architecture` 检查基础目录命名。

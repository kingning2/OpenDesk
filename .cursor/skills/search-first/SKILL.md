---
name: search-first
description: 在实现 OpenDesk 功能前搜索仓库现有能力、契约与边界，避免重复造轮子。
---

# Search First

新增 Feature、crate、Contract、依赖或工具前：

1. 确认调用链只能是 `React → Tauri IPC → Rust`。
2. 搜索现有 `contracts/`、`crates/`、`apps/desktop/src/features/`、`packages/`。
3. 查阅一个匹配的 `skills/opendesk/recipes/` 或 `templates/`。
4. 复用已有 Port、Event、IPC 封装和 `crates/llm`，不要建立平行运行时。
5. 跨端字段按 `Contract → pnpm contracts:sync → Rust → React` 修改。

交付时说明复用了什么、为何需要新增，以及执行了哪些验证。

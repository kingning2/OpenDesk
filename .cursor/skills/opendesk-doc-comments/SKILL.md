---
name: opendesk-doc-comments
description: OpenDesk 简洁中文文档注释与行内注释规范。修改 Rust、TypeScript、React 或订阅代码时使用。
---

# OpenDesk 注释规范

注释的目标是补充类型和代码无法表达的契约、原因与边界。

## 公开 API

- Rust 公开 API 使用简洁中文 rustdoc；说明用途，必要时补充失败条件、不变量或安全边界。
- TypeScript/React 导出 API 仅在用途或约束不明显时写简洁中文 JSDoc。
- 不强制每个 `pub`/`export` 写长模板；显而易见的字段、枚举变体和 re-export 不重复解释。
- 禁止作者、创建日期、机械的参数/返回值章节和复制粘贴模板。

```rust
/// 调用已配置的模型并返回助手文本。
///
/// 网络失败、服务端拒绝或响应不含文本时返回错误。
pub async fn chat_completion(...) -> Result<String, Error> { ... }
```

## 行内注释

- 只解释为什么、兼容性、安全边界或非显然约束。
- 仅复杂多步骤函数使用中文编号分段：

```rust
// 1. 先持久化检查点，避免进程退出后丢失可恢复状态。
// 2. 再发布事件；订阅者此时一定能读取到最新状态。
```

- 简单函数不要编号；禁止 `// 定义变量`、`// 调用接口` 等复述代码。

## 自检

- 注释是否比代码多提供了契约信息？
- 公开边界的失败模式是否清楚？
- 是否删除了过期、冗长或与实现重复的注释？

编码细则见 [coding-standards.md](coding-standards.md)。

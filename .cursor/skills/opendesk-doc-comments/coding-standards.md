# OpenDesk 编码规范

## 结构

- 优先最小可理解实现；有真实状态或生命周期时才引入类型抽象。
- 一个函数做一件事；重复规则提取到共享边界，避免 speculative abstraction。
- Domain 保持纯净，IO 放 Infrastructure，Tauri command 只做边界适配。

## 错误与日志

- Rust 业务路径返回 `Result`，禁止 `unwrap`、`expect`、`panic!`。
- TypeScript 使用 `unknown` 或明确类型，禁止逃逸为 `any`。
- 错误说明操作、原因和可行动信息；禁止吞错。
- 日志带必要标识与耗时，密钥、凭据和敏感正文必须脱敏。

## 注释

- 公开 API 使用简洁中文 rustdoc/JSDoc，写用途与非显然边界。
- 不写作者、日期或固定长模板，不要求为每个 `pub`/`export` 重复类型信息。
- 仅复杂多步骤函数使用 `// 1.`、`// 2.` 中文分段。
- 行内注释解释为什么，禁止翻译代码。

## 验证

按改动范围运行最小测试，并执行：

```bash
pnpm lint
pnpm check:architecture
pnpm contracts:check
```

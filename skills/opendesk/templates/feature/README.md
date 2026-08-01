# Feature Template

一个 Feature 可包含：

```text
contracts/schema/v1/<feature>/
crates/<feature>/
apps/desktop/src/features/<feature>/
```

按 `Contract → codegen → Rust → React` 建立；不存在跨端字段时不要创建空 Contract。Feature 间只用 Event、Query Port 或 Contract。

步骤见 [`../../recipes/add-feature.md`](../../recipes/add-feature.md)。

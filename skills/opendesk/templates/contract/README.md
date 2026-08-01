# Contract Template

复制 [`dto.schema.json.tpl`](dto.schema.json.tpl) 后填写稳定 `$id`、required 与字段约束，再运行：

```bash
pnpm contracts:sync
pnpm contracts:check
```

生成的 Rust 与 TypeScript 文件禁止手改。

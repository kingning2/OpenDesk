# Contract Codegen

将 `contracts/schema/v1/` 中的 JSON Schema 同步为三端类型。

## 运行

```bash
python skills/opendesk/scripts/sync_contracts.py
```

## 输出

| 端 | 目录 |
|----|------|
| TypeScript | `packages/contracts/src/generated/` |
| Rust | `crates/common/src/contracts/` |
| Python | `python/packages/contracts/src/contracts/generated/` |

变更顺序仍为：**Contract → sync_contracts → Rust → Python → React**

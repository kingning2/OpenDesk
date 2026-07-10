# Crate Template

Rust crate 骨架（Feature 或 Infrastructure）。

## Feature Crate

见 [feature/Cargo.toml.tpl](Cargo.toml.tpl) 与 [feature/lib.rs.tpl](lib.rs.tpl)

## Infrastructure Crate

仅导出单一职责模块，无 `app/` / `domain/` 分层。

## TODO

- [ ] 加入根 `Cargo.toml` workspace
- [ ] 在 `crates/app` 组装（若需）
- [ ] `pnpm lint:rust`

## 生成

```bash
python skills/opendesk/scripts/create_crate.py --name <name> --kind feature
```

# Recipe: Add Crate

新增 Rust crate（Feature 或 Infrastructure）。

## 修改顺序

1. `python skills/opendesk/scripts/create_crate.py --name <name> --kind feature|infra`
2. 确认根 `Cargo.toml` `[workspace.members]` 已包含
3. 在 `crates/app` 或 `src-tauri` 注册（若需暴露）
4. `pnpm lint:rust`

## 禁止

- Feature crate 依赖另一 Feature crate
- Infrastructure crate 依赖 Feature crate
- 在 crate 根写业务逻辑

## Checklist

- [ ] `lib.rs` 导出 `app` / `domain`（feature）或单一职责（infra）
- [ ] 无 `unwrap` 于公共 API
- [ ] 命名符合 [naming.md](../guides/naming.md)

## 模板

[../templates/crate/](../templates/crate/)

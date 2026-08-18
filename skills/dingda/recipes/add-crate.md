# Recipe: Add Crate

新增 Rust 基础设施 crate。业务代码直接放在 `apps/desktop/src-tauri`，不创建业务 feature crate。

## 修改顺序

1. `python skills/dingda/scripts/create_crate.py --name <name>`
2. 确认根 `Cargo.toml` `[workspace.members]` 已包含
3. 在 `apps/desktop/src-tauri` 注册（若需暴露）
4. `pnpm lint:rust`

## 禁止

- Infrastructure crate 依赖 Feature 业务代码
- 在 crate 根写业务逻辑

## Checklist

- [ ] `lib.rs` 单一职责导出（无 `app` / `domain` 分层）
- [ ] 无 `unwrap` 于公共 API
- [ ] 命名符合 [naming.md](../guides/naming.md)

## 模板

[../templates/crate/](../templates/crate/)

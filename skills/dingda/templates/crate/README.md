# Crate Template

Rust crate 骨架（仅 Infrastructure）。

## Infrastructure Crate

仅导出单一职责模块，无 `app/` / `domain/` 分层。

业务代码直接放在 `apps/desktop/src-tauri`，不创建业务 feature crate。

## TODO

- [ ] 加入根 `Cargo.toml` workspace
- [ ] 在 `apps/desktop/src-tauri` 组装（若需）
- [ ] `pnpm lint:rust`

## 生成

```bash
python skills/dingda/scripts/create_crate.py --name <name>
```

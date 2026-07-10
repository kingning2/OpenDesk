# Recipe: Add Feature

新增垂直 Feature（如 `chat`）的完整骨架流程。

## 修改顺序

| 步骤 | 操作 | 路径 |
|------|------|------|
| 1 | 定义 Contract 命名空间 | `contracts/schema/v1/<feature>/` |
| 2 | 创建 Rust crate | `crates/<feature>/` |
| 3 | 注册 workspace | 根 `Cargo.toml` |
| 4 | 定义 Query Port（若需跨 Feature 只读） | `crates/ports/src/<feature>_query.rs` |
| 5 | 定义 Event schema（若需跨 Feature 通知） | `contracts/.../event/` |
| 6 | 定义 IPC schema | `contracts/.../ipc/` |
| 7 | 注册 Tauri 命令（空实现） | `apps/desktop/src-tauri/` |
| 8 | 创建前端 Feature 模块 | `apps/desktop/src/features/<feature>/` |
| 9 | 聚合路由 / 侧栏（骨架） | `apps/desktop/src/route/` |
| 10 | Codegen + Lint | scripts |

## 自动化

```bash
python skills/opendesk/scripts/create_feature.py --name <feature>
python skills/opendesk/scripts/sync_contracts.py
python skills/opendesk/scripts/check_architecture.py
pnpm lint
```

## 禁止修改

- 其他 Feature 的内部模块
- `packages/ui` 中添加 IPC
- Python 中添加业务持久化
- 跳过 contracts 直接写 IPC 类型

## 验证

```bash
python skills/opendesk/scripts/check_boundary.py
python skills/opendesk/scripts/check_imports.py
pnpm lint:rust && pnpm lint:frontend
```

## Checklist

- [ ] Contract 命名空间已创建
- [ ] Crate 已加入 workspace
- [ ] 无其他 feature crate 依赖
- [ ] 前端经 platform/ipc，无 @tauri-apps/api
- [ ] 无业务逻辑实现
- [ ] CHANGELOG 已更新（若有 contract）

## 模板

- [../templates/feature/](../templates/feature/)
- [../templates/crate/](../templates/crate/)

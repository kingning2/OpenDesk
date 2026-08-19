# Recipe: Add Feature

新增垂直 Feature（如 `chat`）的完整骨架流程。

## 修改顺序

| 步骤 | 操作 | 路径 |
|------|------|------|
| 1 | 定义 Contract 命名空间 | `contracts/schema/v1/<feature>/` |
| 2 | 业务 UseCase | `apps/desktop/src-tauri/src/<feature>.rs` |
| 3 | 定义 Query Port（若需跨 Feature 只读） | `crates/ports/src/<feature>_query.rs` |
| 4 | 定义 Event schema（若需跨 Feature 通知） | `contracts/.../event/` |
| 5 | 定义 IPC schema | `contracts/.../ipc/` |
| 6 | 注册 Tauri 命令（空实现） | `apps/desktop/src-tauri/` |
| 7 | 创建前端 Feature 模块 | `apps/desktop/src/features/<feature>/` |
| 8 | 聚合路由 / 侧栏（骨架） | `apps/desktop/src/route/` |
| 9 | Codegen + Lint | scripts |

## 自动化

```bash
python skills/dingda/scripts/create_feature.py --name <feature>
python skills/dingda/scripts/sync_contracts.py
python skills/dingda/scripts/check_architecture.py
pnpm lint
```

## 禁止修改

- 其他 Feature 的内部模块
- `packages/ui` 中添加 IPC
- Python 中添加业务持久化
- 未论证就把 Feature 的 AI 放到 Python（ADR-0009）
- 跳过 contracts 直接写 IPC 类型

## 验证

```bash
python skills/dingda/scripts/check_boundary.py
python skills/dingda/scripts/check_imports.py
pnpm lint:rust && pnpm lint:frontend
```

## Checklist

- [ ] Contract 命名空间已创建
- [ ] UseCase 模块已建在 `src-tauri`
- [ ] 基建 crate 无业务代码、业务不依赖基建反向
- [ ] 前端经 platform/ipc，无 @tauri-apps/api
- [ ] 无业务逻辑实现
- [ ] CHANGELOG 已更新（若有 contract）

## 模板

- [../templates/feature/](../templates/feature/)
- [../templates/crate/](../templates/crate/)

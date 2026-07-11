# OpenDesk

企业 AI 客服桌面平台 — Architecture Skeleton 阶段。

## 当前分支职责

**由分支名自动决定。** 运行 `pnpm branch:sync` 生成 [`.cursor/rules/active-branch.mdc`](.cursor/rules/active-branch.mdc)（允许/禁止路径 + 细则规则）。

```bash
pnpm branch:create frontend feature m5-ui-shell   # → frontend/feature/m5-ui-shell
pnpm branch                                     # 交互式
pnpm branch:sync                                # 切换分支后刷新规则
```

| 分支模式 | 职责 |
|----------|------|
| `frontend/<kind>/<slug>` | React · UI · Tauri · `crates/**` |
| `python/<kind>/<slug>` | Python sidecar · `python/**` |
| `contract/<kind>/<slug>` | `contracts/` + codegen |
| `main` | 集成分支 |

配置源：[`skills/opendesk/config/branch_roles.json`](skills/opendesk/config/branch_roles.json)

## 架构约束（硬约束）

```
React（展示）  →  Tauri IPC  →  Rust（协调者）  →  Python（AI Runtime）
```

- React **不知道** Python；Python **不知道** React
- `contracts/` 是跨端 **唯一真相源**，变更顺序：Contract → Codegen → Rust → Python → React
- Feature 完全独立，跨 Feature 只允许 **Query Port**、**Event**、**Contract**

完整约束：[`.cursor/rules/master.md`](.cursor/rules/master.md)

AI 开发知识库：[`skills/opendesk/`](skills/opendesk/)

## 规范入口

- [`.cursor/rules/master.md`](.cursor/rules/master.md) — 全仓库基线
- [`.cursor/rules/branch-workflow.mdc`](.cursor/rules/branch-workflow.mdc) — 分支命令与工作流
- [`.cursor/rules/active-branch.mdc`](.cursor/rules/active-branch.mdc) — **当前分支** scope（生成文件）
- [`.cursor/rules/frontend.md`](.cursor/rules/frontend.md) · [`.cursor/rules/rust.md`](.cursor/rules/rust.md) · [`.cursor/rules/python.md`](.cursor/rules/python.md)

## Code Review 清单

- [ ] 当前分支 scope 内改动？（见 `active-branch.mdc`）
- [ ] 是否跨层？是否跨 Feature？
- [ ] 是否先改了 Contract？
- [ ] 是否违反六边形架构？
- [ ] `pnpm lint` 是否可通过？

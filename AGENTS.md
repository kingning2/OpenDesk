# OpenDesk 分支说明（role/python）

本分支面向 **Developer C（Python/AI Runtime）**。

## 当前阶段

**Architecture Skeleton** — 只允许目录、crate、trait、DTO、Contract、Interface、Mock；**禁止**业务逻辑、Demo、绕架构。

## 架构约束（硬约束）

```
React（展示）  →  Tauri IPC  →  Rust（协调者）  →  Python（AI Runtime）
```

- React **不知道** Python；Python **不知道** React
- `contracts/` 是跨端 **唯一真相源**，变更顺序：Contract → Codegen → Rust → Python → React
- Feature 完全独立，跨 Feature 只允许 **Query Port**、**Event**、**Contract**

完整约束：[`.cursor/rules/master.md`](.cursor/rules/master.md)

AI 开发知识库：[`skills/opendesk/`](skills/opendesk/)（架构 · recipes · templates · scripts）

## 分支

| 分支 | 角色 | 职责 |
|------|------|------|
| `role/frontend` | Developer A | React · `packages/ui` · `packages/platform` |
| `role/rust` | Developer B | Rust · Tauri · `crates/**` |
| `role/python` | Developer C | Python AI Runtime · `python/**` |

## 规范入口

- [`.cursor/rules/master.md`](.cursor/rules/master.md) — 全仓库基线规则（**所有分支必须遵守**）
- [`.cursor/rules/README.md`](.cursor/rules/README.md) — 规则目录说明
- [`.cursor/rules/python.md`](.cursor/rules/python.md) — 本分支规则
- [`contracts/`](contracts/) — 三端共享契约（DTO/IPC/HTTP/Event/Error），任何 Breaking Change 必须先改契约

## Code Review 清单

- [ ] 是否跨层？是否跨 Feature？
- [ ] 是否先改了 Contract？
- [ ] 是否违反六边形架构？
- [ ] 是否符合角色职责？
- [ ] `pnpm lint` 是否可通过？

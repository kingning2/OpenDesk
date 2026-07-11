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

## Managed Docs 变更门禁（所有 Agent 强制）

文档管理入口：[`docs/managed/README.md`](docs/managed/README.md)。详细治理规则以该目录为准，禁止在本文件复制其完整内容。

### 最小上下文读取

处理任何仓库改动时按需渐进读取：

1. `docs/managed/README.md`；
2. `docs/managed/registry/ACTIVE.md`；
3. 与修改路径匹配的一个 Domain 入口；
4. 当前 Change Record；
5. 只有发生冲突或需要追溯设计原因时才读取相关 ADR / 历史 Change。

**禁止**默认递归读取整个 `docs/managed/`。上下文预算遵循 [`docs/managed/CONTEXT_POLICY.md`](docs/managed/CONTEXT_POLICY.md)。

### 先记录，后修改

修改代码、Contract、配置或依赖前，Agent 必须：

1. 从 [`docs/managed/templates/CHANGE.md`](docs/managed/templates/CHANGE.md) 创建独立 Change Record；
2. 将记录加入 [`docs/managed/registry/ACTIVE.md`](docs/managed/registry/ACTIVE.md)；
3. 填写目标、非目标、影响边界和验收标准；
4. 状态至少达到 `approved` 后，才允许开始实现，并切换为 `in_progress`。

紧急修复使用 [`docs/managed/templates/QUICK_FIX.md`](docs/managed/templates/QUICK_FIX.md)，但不得跳过登记。纯只读分析、解释和扫描无需创建 Change Record。

完成修改后必须在同一记录中回填实际结果与验证结论，将状态设为 `completed`，并从 `ACTIVE.md` 移除。若修改稳定领域事实则更新对应 Domain；若形成长期技术决策则创建 ADR。

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

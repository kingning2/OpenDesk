# Workflow Domain

## 职责

建联 **工作流**：阶段定义、路由模板、流程规则、话术片段（与 Mail 模板配合）。

- 工作流阶段 CRUD（stage1~7、归档子状态等，与 Customer `outreach_stage` 联动）
- 工作流模板与账号/路由绑定
- 流程规则引擎（触发条件 → 建议动作，**人确认后执行**）
- 话术库（`script_snippet`）：文件夹、分类、排序
- AI **建议**下一阶段（Agent 只读查客户 → 建议 → UI 确认 → Rust 写入）

## 非职责

- SMTP/IMAP 协议（Mail）
- 客户主数据 CRUD（Customer；本领域只写 `outreach_stage` 相关字段）
- KOL 频道（KOL 领域，暂缓）
- AI 自动改阶段、自动发信

## 稳定边界

```text
React（工作流配置 / 话术库 UI）
  → Tauri IPC（workflow/*）
  → Rust workflow UseCase
  → SQLite（workflow_stage / workflow_template / workflow_rule / script_snippet）
  → 与 Mail 模板渲染、Customer.outreach_stage 协作

AI 阶段建议
  → Agent（只读 Query Port）
  → Rust 校验后人写阶段
```

## 入口

| 类型 | 路径 |
|------|------|
| Rust | `crates/workflow/`（骨架） |
| Contract | `contracts/schema/v1/workflow/`（待建） |
| React | `apps/desktop/src/features/workflow/`（待建） |
| Epic | [EPIC-20260721-001](../../changes/2026/07/epic-20260721-001-email-agent-port.md) Phase 1 |

## 数据模型（规划）

| 表 | 用途 |
|----|------|
| `workflow_stage` | 阶段 id、label、排序、是否归档类 |
| `workflow_template` | 模板正文、适用阶段 |
| `workflow_binding` | 模板与邮箱账号/路由绑定 |
| `workflow_rule` | 流程规则 JSON |
| `script_snippet` | 话术库条目 |

`outreach_stage` 字段归属 **Customer** 表（见 [customer/README.md](../customer/README.md)）。

## 当前状态

**尚未实现。** 仅 crate 骨架；email-agent 对应 `workflow-*.json`、`scripts.json`。

## 当前约束

- 阶段枚举与 email-agent `stages.ts` 对齐，以 **Contract 为唯一定义源**
- 规则引擎产出建议，不自动执行发送或写库

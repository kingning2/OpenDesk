# Alert Domain

## 职责

**预警关键词**维护与历史扫描建议。

- 关键词 CRUD
- 触发 Worker 扫描历史邮件（可选）
- Agent 分析建议 → **人审**后处理

## 非职责

- 自动封禁/自动归档客户（须人确认）
- AI 纠错记忆（Agent 领域 CHG-030，可协作但不重复存储）

## 稳定边界

```text
React（预警关键词 Tab）
  → Tauri IPC（alert/*）
  → Rust alert UseCase
  → SQLite（alert_keyword）
  → Worker 扫描 job → Agent 分析 → UI 展示建议
```

## 入口

| 类型 | 路径 |
|------|------|
| Contract | `contracts/schema/v1/alert/`（待建） |
| React | `apps/desktop/src/features/alerts/`（待建） |
| Epic | [EPIC-20260721-001](../../changes/2026/07/epic-20260721-001-email-agent-port.md) Phase 3~6 |

## 数据模型（规划）

| 表 | 用途 |
|----|------|
| `alert_keyword` | 关键词、分类、启用状态 |

## 当前状态

**尚未实现。** email-agent 对应 `alert-keywords.json`。

## 当前约束

- 扫描结果不自动改 `outreach_stage` 或发送邮件

---
id: CHG-20260720-030-ai-correction-feedback
title: AI 纠错反馈与 Correction Memory
type: change
status: proposed
priority: P0
owner: developer
domain: agent
parent: EPIC-20260720-001-mvp-sales-workbench
depends_on:
  - CHG-20260720-017-ai-readonly-query-tools
  - CHG-20260720-027-llm-provider-settings
blocks: []
milestone: M3
created: 2026-07-20
updated: 2026-07-20
contracts: agent correction IPC + context bundle
related:
  - ADR-0005-ai-correction-memory
---

# AI 纠错反馈与 Correction Memory

## 目标

实现 [ADR-0005](../../decisions/agent/adr-0005-ai-correction-memory.md)：用户标记 AI 错误并保存规则后，**下一次**同类任务自动注入规则，降低重复犯错概率。

完成后可观察到的结果：

- 邮件润色 / WA 建议面板有「标记纠错」流程
- `ai_correction` 表持久化规则（Rust 写入，AI 不写入）
- `context_loader` 在每次 AI 任务前注入匹配的 `corrections[]`
- 设置页可管理全局纠错规则；客户详情可管理客户级规则

## 非目标

- 模型微调、向量检索
- AI 自动创建/修改规则
- 纠错规则跨租户同步

## 背景

用户明确要求：AI 出错后，下次不能犯同样错误。ADR-0005 选定「人工规则 + Rust 注入」方案。

## 影响与边界

### 修改范围

| 层级 | 路径 | 改动内容 |
|------|------|----------|
| Contract | `contracts/schema/v1/agent/dto/ai_correction.schema.json` | 纠错 DTO |
| Contract | `contracts/schema/v1/agent/dto/agent_context_bundle.schema.json` | customer + pricing + corrections |
| Contract | `contracts/schema/v1/agent/dto/correction_category.schema.json` | 分类枚举 |
| Contract | `contracts/schema/v1/agent/ipc/correction_save.request.schema.json` | 人提交纠错 |
| Contract | `contracts/schema/v1/agent/ipc/correction_list.response.schema.json` | 列表（可按 customer/task 过滤） |
| Contract | `contracts/schema/v1/agent/ipc/correction_toggle.request.schema.json` | 启用/停用 |
| Storage | Migration `create_ai_correction_table` | 见 database-schema §5.x |
| Rust | `crates/agent/src/correction/`（新建） | CRUD UseCase（**仅 IPC 写**） |
| Rust | `crates/agent/src/app/context_loader.rs` | 合并 corrections 进 bundle |
| Rust | `crates/agent/src/app/run_task.rs` | 任务前调用 context_loader |
| Python | `python/packages/agent/agent/orchestrator/context_loader.py` | 消费 bundle.corrections |
| Python | `python/packages/agent/agent/prompts/corrections_block.py` | 渲染 MUST FOLLOW 区块 |
| Frontend | `apps/desktop/src/features/agent/correction-report-dialog.tsx` | 标记纠错 UI |
| Frontend | `apps/desktop/src/features/agent/correction-rules-panel.tsx` | 规则管理 |
| Frontend | `apps/desktop/src/features/mail/ai-draft-panel.tsx` | 集成「标记纠错」 |
| Frontend | `apps/desktop/src/features/channel/suggest-reply-panel.tsx` | 集成「标记纠错」 |
| Frontend | `apps/desktop/src/features/setting/` 或 customer 详情 | 规则列表入口 |
| Docs | `docs/managed/domains/agent/README.md` | Correction Memory 说明 |

### 不修改范围

| 路径 | 原因 |
|------|------|
| Python 写 `ai_correction` | **禁止** |
| Agent Tool `correction.save` | 写操作仅 UI→Rust IPC |

### Contract

- **需要** correction + context_bundle schema

### 跨层

- React → Rust 写纠错；Rust → Python 读 bundle（含 corrections）

## 依赖关系

- 前置：CHG-017（context_loader 骨架）、CHG-027（LLM 可用）
- 与 CHG-018、CHG-020 **同期或紧随其后**（面板集成）

## 实施方案

1. Migration `ai_correction` 表 + 索引（customer_id, task_type, is_active）。
2. `agent.correction_save`：校验 scope/customer_id/rule_text 非空。
3. 扩展 `context_loader`：查询匹配规则，应用 ADR-0005 限额。
4. mail_draft / wa_suggest 任务 payload 使用 `AgentContextBundle`。
5. UI：生成结果旁「标记纠错」→ 对话框（category、rule、scope、可选 bad/good 示例）。
6. 设置页「AI 纠错规则」：列表、开关、删除。

## 验收

- [ ] 可对 mail_draft 结果保存一条 customer 级规则
- [ ] 同一客户再次生成时，Rust 日志/调试可见 corrections 注入（或 Python 返回 metadata）
- [ ] 全局规则对 wa_suggest 生效
- [ ] 停用规则后不再注入
- [ ] Python 无 ai_correction 写 API / SQLite
- [ ] 架构检查通过

## 实际结果

（完成前留空。）

## 后续项

- 编辑 diff 自动生成 rule_text 建议
- 规则过期 `expires_at`

---
id: ADR-0005-ai-correction-memory
title: AI 纠错记忆（人工规则注入，非模型写库）
type: adr
status: accepted
domain: agent
created: 2026-07-20
updated: 2026-07-20
deciders: product-owner
related:
  - ADR-0001-ai-readonly-query-port
  - CHG-20260720-030-ai-correction-feedback
  - EPIC-20260720-001-mvp-sales-workbench
---

# AI 纠错记忆（人工规则注入，非模型写库）

## Status

Accepted — MVP 必须实现，见 [CHG-030](../../changes/2026/07/chg-20260720-030-ai-correction-feedback.md)。

## Context

AI 在邮件润色、WhatsApp 建议等场景会犯错（错误币种、忽略客户报价、语气不当、捏造事实）。用户要求：**下次不要犯同样的错误**。

约束（与 ADR-0001 一致）：

- Python **不能**写 SQLite，不能自行「记住」纠错
- 不能依赖模型隐式记忆（会话结束即失效）
- MVP 不做微调 / 向量 RAG 平台
- 纠错规则须可审计、可关闭、可限定客户范围

## Decision

采用 **「人工纠错记录 → Rust 注入上下文」** 机制，简称 **Correction Memory**。

### 1. 数据模型：`ai_correction`

由 **人通过 UI 创建**，Rust UseCase 写入 `opendesk.db`；AI 只读消费（经 Rust 打包进任务 payload，**不**作为 Agent Tool 让 Python 随意查库）。

| 字段 | 说明 |
|------|------|
| `id` | UUID |
| `scope` | `global` \| `customer` |
| `customer_id` | scope=customer 时必填 |
| `task_type` | `mail_draft` \| `wa_suggest` \| `wa_translate` \| `all` |
| `category` | `wrong_price`, `wrong_currency`, `wrong_tone`, `wrong_language`, `factual_error`, `ignored_context`, `format_issue`, `custom` |
| `rule_text` | **祈使句规则**，如「该客户报价一律使用 EUR，禁止写 USD」 |
| `example_bad` | 可选，错误片段 |
| `example_good` | 可选，正确片段 |
| `source_task_id` | 可选，关联触发纠错的 Agent 任务 |
| `is_active` | 默认 true |
| `created_by` | 本地操作者 |
| `created_at` | ISO-8601 |

### 2. 捕获流程（仅人触发）

```text
AI 生成草稿 → 用户审阅
  ├─ 满意 → 发送/采用
  └─ 不满意 →「标记纠错」
        ├─ 选 category
        ├─ 写 rule_text（可一键从编辑 diff 生成建议）
        ├─ 选 scope：仅本客户 / 全局（同类任务）
        └─ Rust 保存 ai_correction
```

**可选增强：** 用户大幅编辑 AI 正文后发送前，若编辑距离超过阈值，轻提示「是否保存为纠错规则？」（不强制）。

### 3. 注入流程（Rust `context_loader`）

每次 AI 任务（mail_draft / wa_suggest / wa_translate）**开始前**，Rust 组装 `AgentContextBundle`：

```json
{
  "customer": { ... },
  "pricing": { ... },
  "corrections": [
    {
      "category": "wrong_currency",
      "rule_text": "该客户报价一律使用 EUR，禁止写 USD",
      "example_bad": "... USD 99 ...",
      "example_good": "... EUR 99 ..."
    }
  ]
}
```

**匹配规则（按优先级合并，去重）：**

1. `scope=customer` AND `customer_id` 匹配 AND (`task_type` 匹配 OR `all`)
2. `scope=global` AND (`task_type` 匹配 OR `all`)
3. 仅 `is_active=1`，按 `created_at DESC`

**限额（防止 prompt 爆炸）：**

- 最多 **15** 条规则
- `rule_text` 合计不超过 **3000** 字符
- 超出时优先保留 customer 级，再 global 最新

Python 将 `corrections` 渲染进 system prompt 固定区块：

```text
## MUST FOLLOW — Prior corrections (do not repeat these mistakes)
- [wrong_currency] 该客户报价一律使用 EUR，禁止写 USD
...
```

Python **不得**修改、删除 corrections；不得写回 `ai_correction` 表。

### 4. 管理与验证

- 设置页 / Agent 子页：**纠错规则列表**（启用/停用/删除）
- 客户详情可查看 **仅该客户** 的规则
- 验收：保存规则后，对同一客户再次生成，prompt 含该规则；生成结果应遵守（人审确认）

### 5. 明确不做（MVP）

| 方案 | 原因 |
|------|------|
| 模型微调 | 成本高、难审计 |
| 向量 RAG 纠错库 | 超出 MVP；结构化规则更可控 |
| AI 自动写纠错 | 违反「人维护、AI 只读」 |
| Python Tool `correction.save` | 扩大写权限面 |

## Alternatives Considered

| 方案 | 不选原因 |
|------|----------|
| 仅靠更长 system prompt | 无法积累客户级经验 |
| 把纠错塞进 customer.notes | 非结构化，难按任务类型过滤 |
| 会话内 ChatGPT 式 memory | 换会话/重启即丢失 |

## Consequences

### Positive

- 可审计、可关闭、可 per-customer
- 符合 Rust 协调者与 ADR-0001
- 不引入 RAG 基础设施

### Negative

- 依赖用户愿意点「标记纠错」；需 UI 引导
- 规则过多时须人工整理（限额 + 列表管理）

## Compliance

- CHG-030 实现本 ADR
- CHG-017 `context_loader`、CHG-018/020 任务入口须注入 corrections
- `check_architecture.py` 后续可加：Python 无 `ai_correction` 写路径

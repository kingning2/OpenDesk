# Agent Domain

## 职责

Agent 领域负责 AI 规划、模型交互、结构化工具请求和任务执行状态，不负责业务持久化或绕过 Rust 执行高权限操作。

MVP 增量能力：

- **LLM Provider 配置**（设置页厂商预设 + API Key 存 OS keyring；IPC **永不**回传密钥，仅 `has_api_key` / `configured`）— 见 [CHG-027](../../changes/2026/07/chg-20260720-027-llm-provider-settings.md)（completed）
- **只读 Query 工具**（customer.* / pricing.* / quote.history / ocr.get_text）— 见 ADR-0001
- **Correction Memory**（人工纠错规则，Rust 注入下次任务，见 [ADR-0005](../../decisions/agent/adr-0005-ai-correction-memory.md)、[CHG-030](../../changes/2026/07/chg-20260720-030-ai-correction-feedback.md)）
- **邮件润色**（`mail_draft`）— 在 **Mail 模板已填充** 的正文上个性化，人审后发
- **WhatsApp 翻译与回复建议**（wa_translate / wa_suggest）— 人发

## 非职责

- 直接修改客户、报价、合作状态（**禁止写库工具**）
- **AI 自动写入纠错规则**（仅人通过 UI → Rust IPC）
- 触发 SMTP 或 WhatsApp 发送
- Python 直连 SQLite
- 业务持久化（`ai_correction` 由 Rust 写入）

## Correction Memory 摘要

```text
用户标记 AI 错误 → UI → Rust 存 ai_correction
下次 mail_draft / wa_suggest → Rust context_loader 注入 corrections[]
→ Python 渲染 MUST FOLLOW 区块 → 生成（人审）
```

| 作用域 | 说明 |
|--------|------|
| `customer` | 仅该客户任务生效 |
| `global` | 同 task_type 全部生效 |

限额：最多 15 条 / 3000 字符（见 ADR-0005）。

## 目标边界

- Python 产生结构化 `ToolCall`（**仅只读白名单**）；
- Rust 负责审批、授权和工具执行；
- Python 接收 `ToolResult` 后恢复任务；
- 长任务使用 `task_id`，支持暂停、恢复、取消和超时；
- 生成邮件/WA 建议前 **必须** 经 `context_loader` 拉取客户上下文 **与纠错规则**。

## 稳定边界

```text
React → Rust agent IPC → Python sidecar task
         ↑ correction_save（人写）
         ↑ context_loader（customer + pricing + corrections）
                              ↓ ToolCall（只读）
                         Rust Query Port → SQLite
                              ↓ ToolResult
                         Python 生成 → Rust → React
```

**有效 ADR：**

- [ADR-0001-ai-readonly-query-port](../../decisions/customer/adr-0001-ai-readonly-query-port.md)
- [ADR-0005-ai-correction-memory](../../decisions/agent/adr-0005-ai-correction-memory.md)

## 入口

- Rust：`crates/agent/`
- Python：`python/packages/agent/`
- Contract：`contracts/schema/v1/agent/`
- Epic：[EPIC-20260720-001-mvp-sales-workbench](../../changes/2026/07/epic-20260720-001-mvp-sales-workbench.md)

## 当前状态

LLM Provider 配置（CHG-027）已落地：设置弹窗「AI / LLM」、`keyring` 存密钥、Sidecar `llm_test_connection`。其余 Agent 能力见 CHG-017、CHG-018、CHG-020、**CHG-030**（仍为骨架 / 规划）。

新的 Agent 内核设计应先创建 Change Record；形成长期技术选择时再建立 ADR（只读查库 ADR-0001；纠错记忆 ADR-0005）。

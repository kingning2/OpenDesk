---
id: CHG-20260720-017-ai-readonly-query-tools
title: AI 只读 Query Port 与 Agent 工具
type: change
status: proposed
priority: P0
owner: developer
domain: agent
parent: EPIC-20260720-001-mvp-sales-workbench
depends_on:
  - CHG-20260720-013-customer-profile-model
  - CHG-20260720-016-pricing-catalog
  - CHG-20260720-027-llm-provider-settings
blocks:
  - CHG-20260720-018-ai-mail-draft
  - CHG-20260720-020-whatsapp-business-assist
  - CHG-20260720-028-ocr-business-scenarios
milestone: M3
created: 2026-07-20
updated: 2026-07-20
contracts: agent tool call + query DTO
related:
  - ADR-0001-ai-readonly-query-port
---

# AI 只读 Query Port 与 Agent 工具

## 目标

实现 ADR-0001：Python Agent 通过 Rust **白名单只读工具** 查询客户与价目表；**无任何写库工具**；Python 不连接 SQLite。

完成后 AI 可精确获取：客户来源、报价、合作状态、价目表、报价历史、沟通摘要。

## 非目标

- AI 邮件/WA 正文生成（CHG-018 / CHG-020）
- 写库工具
- 任意 SQL 工具
- 前端 Query UI

## 背景

用户明确要求 AI 有查库权限、无改库权限。Must 在 Agent 层落地，而非 Prompt 里塞全表 JSON。

## 影响与边界

### 修改范围

| 层级 | 路径 | 改动内容 |
|------|------|----------|
| Contract | `contracts/schema/v1/agent/dto/tool_call.schema.json` | ToolCall 结构（若无则建） |
| Contract | `contracts/schema/v1/agent/dto/tool_result.schema.json` | ToolResult |
| Contract | `contracts/schema/v1/agent/tools/customer_get.request.schema.json` | 工具参数 |
| Contract | `contracts/schema/v1/agent/tools/customer_search.request.schema.json` | 搜索参数 |
| Contract | `contracts/schema/v1/agent/tools/customer_timeline.request.schema.json` | 时间线参数 |
| Contract | `contracts/schema/v1/agent/tools/pricing_list.request.schema.json` | 价目表 |
| Contract | `contracts/schema/v1/agent/tools/pricing_match.request.schema.json` | 阶梯匹配 |
| Contract | `contracts/schema/v1/agent/tools/quote_history.request.schema.json` | 报价史 |
| Contract | `contracts/schema/v1/agent/tools/ocr_get_text.request.schema.json` | OCR 文本只读（CHG-028） |
| Contract | `contracts/schema/v1/agent/sidecar/query_tools.request.schema.json` | Rust→Python 或 Python→Rust 工具往返 |
| Rust | `crates/agent/src/query_port/`（新建） | 只读 Query Port 实现 |
| Rust | `crates/agent/src/query_port/whitelist.rs` | 工具白名单校验 |
| Rust | `crates/agent/src/app/execute_tool.rs` | 路由 ToolCall → Query Port |
| Rust | `crates/customer/src/query/` | customer.get/search/timeline/quote_history |
| Rust | `crates/pricing/src/query/` | pricing.list/match |
| Python | `python/packages/agent/agent/tools/readonly/`（新建） | 工具定义与 handler 注册 |
| Python | `python/packages/agent/agent/orchestrator/context_loader.py`（新建） | 生成前拉客户上下文 |
| Docs | `docs/managed/domains/agent/README.md` | 补充只读工具说明 |

### 不修改范围

| 路径 | 原因 |
|------|------|
| `crates/mail-net/**` | 发送不由工具触发 |
| `crates/channel/**` | WA 在 CHG-020 |
| Customer/ Pricing **写** UseCase | 本 Change 只加 query 模块 |
| Python SQLite 驱动 | **禁止引入** |

### Contract

- **需要** agent tools + query 往返 schema

### 跨层

- Python ToolCall → Rust execute → SQLite read → ToolResult → Python

### 跨 Feature

- Agent crate 依赖 customer/pricing query 模块（Rust 层）

### 风险

- 须单元测试：注册工具列表中无 write 类工具名
- 返回 DTO 须脱敏（不含内部 id 以外的敏感凭据）

## 依赖关系

- 父任务：EPIC-20260720-001
- 前置任务：CHG-013、CHG-016
- ADR：ADR-0001（accepted）
- 阻塞：CHG-018、CHG-020

## 实施方案

1. Contract 定义 6 个只读工具及 DTO。
2. Rust Query Port：参数校验 → Repository 只读查询 → 脱敏 DTO。
3. `whitelist.rs` 硬编码允许工具 id；未知 id 拒绝。
4. Python：注册对称工具名；handler 经 sidecar IPC 调 Rust execute_tool。
5. `context_loader`：`customer_id` 必填时依次拉 customer.get、quote.history、pricing.list。
6. 测试：尝试 register `customer.update` 须失败；Python 侧无 sqlite3 import。

## 验收

- [ ] 6 个只读工具均可返回合法 DTO
- [ ] 无 write/update/delete 工具可注册
- [ ] Python 代码库无 SQLite 连接
- [ ] context_loader 缺 customer_id 时拒绝
- [ ] ADR-0001 验收项全部满足
- [ ] 架构检查通过

## 实际结果

（完成前留空。）

## 后续项

- [CHG-018 AI 邮件起草](chg-20260720-018-ai-mail-draft.md)

# Workflow Domain

## 状态

**planned（暂缓）** — 本领域原为 email-agent 商务建联工作流（阶段、话术库、规则引擎），依赖已移除的 **Mail / Customer 板块**。本期不实现；若未来重新引入 Mail/Customer，按新 Change 重新规划。

## 职责（未来）

- 工作流阶段 CRUD 与路由模板
- 流程规则引擎（触发 → 建议动作，**人确认后执行**）
- 话术库（`script_snippet`）
- AI **建议**下一阶段（只读查询 → 建议 → UI 确认 → Rust 写入）

## 非职责

- SMTP/IMAP 协议（Mail 板块已移除）
- 客户主数据（Customer 板块已移除）
- AI 自动改阶段、自动发信

## 当前状态

**未实现。** `crates/workflow/` 无骨架；Email-Agent 移植项目已移除，重新引入时按新 Change 设计。

## 当前约束

- 规则引擎产出建议，不自动执行发送或写库

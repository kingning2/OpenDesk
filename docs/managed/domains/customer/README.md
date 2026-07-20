# Customer Domain

## 职责

客户档案（Lead/Customer）的全生命周期数据：**来源、联系方式、商务状态、报价、正式合作字段、沟通时间线**。

- 客户 CRUD（仅 Rust + UI，**非 AI**）
- 报价变更历史与审计
- 合作信息（套餐、月费、合约起止）
- 沟通时间线（邮件/WA/人工备注）
- 为 AI 提供只读 Query Port 的数据源（见 ADR-0001）

## 非职责

- SMTP 发信协议（Mail 领域）
- YouTube 爬取逻辑（Crawler 领域）
- WhatsApp API 接入（Channel 领域）
- 价目表内容维护的业务规则细节（Pricing 领域，但客户引用报价结果）
- LLM 推理与 Prompt 组装（Agent / Python Runtime）
- AI 写库或改客户状态

## 稳定边界

```text
React（客户详情/编辑 UI）
  → Tauri IPC（customer/*）
  → Rust UseCase（写）/ Query Port（AI 只读）
  → SQLite（customer / quote_history / timeline / cooperation 表）
  → AI 仅经 Query Port 读取，经 ADR-0001 白名单工具
```

**输入：**

- 爬虫导入、人工新建/编辑
- Mail/Channel 发收信时 Rust 写入时间线

**输出：**

- 客户 DTO 给前端列表/详情
- 只读 DTO 给 AI ToolResult
- 事件（可选）：`customer.updated` 供 UI 刷新

## 入口

| 类型 | 路径（规划） |
|------|--------------|
| Rust crate | `crates/` 下新建 `customer` 或扩展现有 `user`（Epic 实施时定） |
| 存储 | `crates/storage/src/opendesk_db/`（`customer` 等表，见 [database-schema.md](../../../architecture/database-schema.md)） |
| Contract | `contracts/schema/v1/customer/` |
| React Feature | `apps/desktop/src/features/customer/`（新建） |
| ADR | [ADR-0001-ai-readonly-query-port](../../decisions/customer/adr-0001-ai-readonly-query-port.md) |
| Epic | [EPIC-20260720-001-mvp-sales-workbench](../../changes/2026/07/epic-20260720-001-mvp-sales-workbench.md) |

## 客户数据模型（MVP 目标）

### 主表 `customer`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | UUID | 主键 |
| `display_name` | text | 显示名/称呼 |
| `email` | text | 唯一索引，主联系邮箱 |
| `whatsapp_phone` | text? | 可选，M5 绑定 WA |
| `source_channel` | enum | MVP 固定 `youtube` |
| `source_meta` | JSON | 频道 id、URL、标题等 |
| `lifecycle_status` | enum | `new` / `contacted` / `negotiating` / `won` / `lost` / `paused` |
| `quoted_price` | decimal? | 当前报价 |
| `quoted_currency` | text? | 币种 |
| `quoted_at` | datetime? | 当前报价日期 |
| `pricing_tier` | text? | 命中阶梯名称 |
| `cooperation_status` | enum | `none` / `negotiating` / `active` / `paused` / `terminated` |
| `package_name` | text? | 合作套餐名 |
| `monthly_fee` | decimal? | 月费 |
| `contract_start` | date? | 合作开始 |
| `contract_end` | date? | 合作结束 |
| `notes` | text? | 商务备注 |
| `created_at` / `updated_at` | datetime | 审计 |

### 子表

| 表 | 用途 |
|----|------|
| `quote_history` | 报价变更：旧价、新价、操作人、时间、原因 |
| `customer_timeline` | 沟通摘要：类型（email_sent/wa_in/wa_out/note）、引用 id、摘要文本 |
| `cooperation_audit` | 合作字段变更审计（可选与 quote_history 合并策略，实施时定） |

## 当前状态

**尚未实现。** 仓库仅有 `crates/user`、`crates/tenant` 骨架，无客户 CRM 表与 UI。

实施顺序见 [CHG-013](../../changes/2026/07/chg-20260720-013-customer-profile-model.md)。

## 当前约束

- AI **不得**修改本领域任何表（ADR-0001）
- 邮箱在 MVP 内 **全局唯一**（去重键）
- 所有写操作须记录 `updated_at`；报价/合作变更须写历史表

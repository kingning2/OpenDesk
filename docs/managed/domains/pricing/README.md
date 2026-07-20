# Pricing Domain

## 职责

企业 **价目表与阶梯报价** 的维护与查询，供 AI 起草邮件/WA 建议时引用。

- 套餐/产品定义
- 阶梯规则（数量档、地区、时长等——MVP 可先简化）
- 对外只读查询（含 AI `pricing.list` / `pricing.match`）
- 管理 UI（增删改套餐与阶梯）

## 非职责

- 客户个体报价的最终裁定（Customer 领域 `quoted_price` + `quote_history`）
- 订单计费、发票、支付
- AI 自动改价或自动发送报价
- Python 直连价目表存储

## 稳定边界

```text
React（价目表管理 UI）
  → IPC pricing/*
  → Rust UseCase（写）/ Query Port（AI 只读）
  → SQLite pricing 表

AI Agent
  → pricing.list / pricing.match（只读，ADR-0001）
  → ToolResult → Prompt 注入
```

**与客户领域关系：**

- Pricing：公司级价目表（「标准价」）
- Customer：该客户当前报价、历史议价（「成交价/意向价」）
- AI 起草须 **同时** 读取价目表与客户报价字段

## 入口

| 类型 | 路径（规划） |
|------|--------------|
| Rust | `crates/` 新建 `pricing` 或并入 `customer`（实施时定） |
| 存储 | `crates/storage/src/pricing_db/` |
| Contract | `contracts/schema/v1/pricing/` |
| React | `apps/desktop/src/features/pricing/` 或 settings 子页 |
| Epic 子任务 | [CHG-016](../../changes/2026/07/chg-20260720-016-pricing-catalog.md) |

## 数据模型（MVP 目标）

### `pricing_package`

| 字段 | 说明 |
|------|------|
| `id` | 主键 |
| `name` | 套餐名 |
| `description` | 说明 |
| `currency` | 默认币种 |
| `is_active` | 是否启用 |
| `sort_order` | 排序 |

### `pricing_tier`

| 字段 | 说明 |
|------|------|
| `id` | 主键 |
| `package_id` | 外键 |
| `tier_name` | 阶梯名 |
| `min_quantity` / `max_quantity` | 数量区间（可空表示不限） |
| `unit_price` / `monthly_fee` | 单价或月费 |
| `conditions_json` | 扩展条件（地区等） |

MVP 可支持 **手动 CSV/JSON 导入** 作为快捷方式，但须有 Rust 校验与 UI 展示。

## 当前状态

**尚未实现。** 无 pricing crate、Contract 或 UI。

## 当前约束

- AI 仅只读访问（ADR-0001）
- 价目表变更不影响已有 `quote_history` 历史记录
- 匹配逻辑须在 Rust 实现，Python 不得自行「猜价」

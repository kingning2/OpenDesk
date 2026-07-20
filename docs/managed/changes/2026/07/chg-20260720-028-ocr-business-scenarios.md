---
id: CHG-20260720-028-ocr-business-scenarios
title: OCR 商务场景定义与客户关联
type: change
status: proposed
priority: P1
owner: developer
domain: ocr
parent: EPIC-20260720-001-mvp-sales-workbench
depends_on:
  - CHG-20260720-024-ocr-worker-pipeline
  - CHG-20260720-013-customer-profile-model
  - CHG-20260720-017-ai-readonly-query-tools
blocks: []
milestone: M6
created: 2026-07-20
updated: 2026-07-20
contracts: ocr scenario DTO + agent ocr_get tool
related:
  - ADR-0002-heavy-work-worker-process
  - ADR-0003-tesseract-local-model-on-demand-download
---

# OCR 商务场景定义与客户关联

## 目标

MVP **必须**明确 OCR 在商务工作台中的业务场景（非纯技术演示），并完成 UI/数据/AI 只读联调。完成后：

| 场景 ID | 用户动作 | 产出 |
|---------|----------|------|
| `biz_card` | 扫描/导入名片图，选客户或新建 | 识别姓名/邮箱/电话；可一键填入客户表单 |
| `contract_snippet` | 上传合同/报价单截图，绑 customer_id | OCR 全文入库；timeline `ocr_completed` |
| `email_screenshot` | 粘贴客户邮件截图 | 绑 customer；辅助「记录客户回复」预填正文（可选） |
| `price_list_image` | 上传价目表图片 | 识别文本供人核对价目表录入（不自动写 pricing 表） |

AI 可通过只读工具 `ocr.get_text`（按 `ocr_job_id` 或 `customer_id` 最近一条）获取识别文本，用于邮件润色/WA 建议上下文。

## 非目标

- OCR 结果自动改客户报价/合作状态（须人确认）
- 名片自动建档（MVP 仅预填表单，人点保存）
- 表格结构化解析（发票/复杂表格 MVP 后）
- 云 OCR

## 背景

M6 若只交付「能识别文字」不足以服务商务闭环。用户已确认 MVP 文档须写清 OCR 业务场景与主链路关系。

## 影响与边界

### 修改范围

| 层级 | 路径 | 改动内容 |
|------|------|----------|
| Contract | `contracts/schema/v1/ocr/dto/scenario_type.schema.json` | 上表四类枚举 |
| Contract | `contracts/schema/v1/ocr/ipc/submit.request.schema.json` | 增加 `scenario_type` |
| Contract | `contracts/schema/v1/agent/tools/ocr_get_text.request.schema.json` | 只读工具 |
| Rust | `crates/ocr/src/domain/scenario.rs` | 场景元数据 |
| Rust | `crates/ocr/src/app/extract_biz_card.rs` | 简单正则抽邮箱/电话（启发式） |
| Rust | `crates/agent/src/query_port/ocr.rs` | `ocr.get_text` 白名单 |
| Frontend | `apps/desktop/src/features/ocr/ocr-submit-wizard.tsx` | 场景选择 + 客户绑定 |
| Frontend | `apps/desktop/src/features/customer/biz-card-import-dialog.tsx` | 名片预填 |
| Frontend | `apps/desktop/src/features/ocr/ocr-result-panel.tsx` | 按场景展示动作（复制/预填/记时间线） |
| Docs | `docs/managed/domains/ocr/README.md` | 商务场景专节（本 Change 同步） |

### 不修改范围

| 路径 | 原因 |
|------|------|
| Worker Tesseract 核心 | CHG-024 |
| pricing 表自动写入 | 人维护 |

### Contract

- **需要** scenario_type + agent ocr_get_text

### 跨 Feature

- OCR → Customer 预填/绑定；Agent 只读 Query Port

## 依赖关系

- 前置：CHG-024（管线）、CHG-013（客户）、CHG-017（只读工具扩展）
- 父任务：EPIC-20260720-001

## 实施方案

1. 提交 OCR 时必选 `scenario_type`；`customer_id` 除 `biz_card` 外建议必填。
2. `biz_card`：识别完成后弹窗展示抽取字段，跳转客户新建/编辑预填。
3. `contract_snippet` / `email_screenshot`：完成后写 timeline；`email_screenshot` 可提供「填入记录回复」快捷操作（调 CHG-026）。
4. `price_list_image`：结果页提示「请对照价目表设置手动录入」。
5. CHG-017 扩展白名单：`ocr.get_text`（限 customer 已绑定 job）。

## 验收

- [ ] 四类场景均可在 UI 走完提交 → 结果 → 关联动作
- [ ] `biz_card` 可预填客户表单邮箱/电话
- [ ] 绑 customer's OCR 出现在 timeline
- [ ] `ocr.get_text` 只读工具可用且无写库
- [ ] 领域文档含完整场景说明

## 实际结果

（完成前留空。）

## 后续项

- 表格 OCR / 发票结构化
- OCR 结果 RAG 索引（MVP 后）

---
id: ROADMAP-mvp-sales-workbench
title: MVP 商务工作台路线图
type: roadmap
status: active
domain: product
updated: 2026-07-20
epic: EPIC-20260720-001-mvp-sales-workbench
---

# MVP 商务工作台路线图

## 1. 文档定位

本文件是 **OpenDesk MVP 的总路线图**，回答：

- 产品真正要交付什么（不是 WhatsApp 自动客服）；
- 按什么顺序推进；
- 每份相关文档负责什么、改动边界在哪里。

**不承载**：单次变更的实施细节、完整数据表 DDL、逐文件 diff。这些分别放在 Epic、Change Record、Domain、ADR 中。

## 2. 产品定义（已确认）

### 2.1 一句话

YouTube 爬邮箱获客 → 用自己的 SMTP 邮件谈价 → WhatsApp Business 在桌面收发（人发，AI 翻译/建议）→ 每位客户的来源、报价、合作状态被系统记住，AI 通过 **只读查询** 获取客户上下文。

### 2.2 核心用户流程

```text
1. 爬虫从 YouTube 频道简介提取邮箱；**无邮箱则 Worker Playwright 打开 About 页补全（不丢弃频道）**
2. 邮箱进入客户档案（来源、频道元数据）
3. 销售在桌面端 **选择邮件模板** → 变量填充 → 对该客户 SMTP 发出
4. AI 在模板基础上根据价目表 + 客户档案 + **历史纠错规则** **润色** 邮件（人审后发）
5. **IMAP 自动收取**客户邮件回复，进入时间线
6. WhatsApp Business 消息进入桌面；AI 翻译并建议回复（人点发送）
7. AI 犯错时用户 **标记纠错**；下次同类任务 Rust 自动注入规则
8. 报价、合作状态由人在 UI 维护；AI 只能查，不能改
```

### 2.3 MVP 成功标准

| # | 标准 | 验收方式 |
|---|------|----------|
| S1 | 爬虫邮箱可建档为客户，按邮箱去重 | 同一邮箱不重复建档；详情页可见 YouTube 来源；**无邮箱频道保留，Playwright 补全后可导入** |
| S2 | 绑定 SMTP + **邮件模板**，对客户发信并留痕 | 选模板→渲染→发送成功；timeline 含 template |
| S2b | 内置模板覆盖首联/跟进/报价/改价/确认合作 | 各 intent 至少 1 份可用 |
| S2c | 客户邮件回复出现在时间线 | `email_received`；手动兜底（CHG-026） |
| S2d | **IMAP 自动收信**匹配客户邮箱 | Worker 同步；Message-ID 去重；未匹配可人工关联 |
| S3 | 客户档案含正式合作字段（套餐、月费、起止、报价史） | 详情页字段完整；变更写入历史 |
| S4 | AI 起草邮件前必须注入该客户档案 + 价目表 | 缺客户 id 或关键字段时拒绝生成并提示 |
| S5 | AI 仅通过 Rust 白名单只读工具查库 | Python 无 SQLite 连接；无 write/update/delete 工具 |
| S6 | WhatsApp 桌面收发 + 翻译/建议，不自动回复 | 发送必须人工触发 |
| S6b | WhatsApp webhook 开发环境可联调 | 隧道 + 测试号入站消息可在 UI 显示 |
| S7 | OCR 提交后 UI 不卡顿 | 主进程无 OCR CPU 负载；任务在 Worker |
| S8 | OCR 结果可读、可关联客户 | ocr_text_block + 可选 timeline |
| S9 | Tesseract 语言包用户按需下载 | 安装包无 tessdata；UI 点击下载后可用 |
| S10 | LLM Provider 可在设置页配置并测试连接 | 未配置时 AI 功能明确阻断 |
| S11 | OCR 四类商务场景可走通 | 名片/合同截图/邮件截图/价目图 |
| S12 | AI 纠错规则保存后下次任务注入 | 同客户/同任务类型不再重复同类错误（人审确认） |

### 2.4 明确不做（MVP 范围外）

- WhatsApp / 邮件 **自动发送**、无人值守客服
- AI **写库**（改报价、改合作状态、删客户）；**纠错规则仅人写、Rust 注入**
- 多渠道获客（除 YouTube 外）
- 订单/库存/ERP Tool Calling
- 多租户 RBAC、私有化部署全套
- 完整 RAG 知识库平台

## 3. 文档地图（每份文档做什么）

| 文档 | 路径 | 职责 | 何时读 |
|------|------|------|--------|
| **本路线图** | `roadmaps/mvp-sales-workbench.md` | 里程碑顺序、成功标准、文档索引 | 规划任何 MVP 工作前 |
| **Epic** | `changes/2026/07/epic-20260720-001-mvp-sales-workbench.md` | 总目标、子任务关系、总体验收 | 拆任务、看依赖 |
| **ADR：AI 只读查库** | `decisions/customer/adr-0001-ai-readonly-query-port.md` | AI 不得直连 DB；只读 Query Port 长期约束 | 做 Agent 工具、Python 查询前 |
| **ADR：AI 纠错记忆** | `decisions/agent/adr-0005-ai-correction-memory.md` | 人工规则注入，防重复犯错 | 做 CHG-030 / M3 AI 前 |
| **ADR：WA Webhook 部署** | `decisions/channel/adr-0004-whatsapp-webhook-deployment.md` | 开发隧道与生产 relay 选项 | 做 CHG-020 / M5 前 |
| **Domain：Customer** | `domains/customer/README.md` | 客户档案、报价史、合作字段、时间线边界 | 改客户模型/UI 前 |
| **Domain：Mail** | `domains/mail/README.md` | **邮件模板**、SMTP 发信、变量渲染边界 | 改邮件功能前 |
| **Domain：Crawler** | `domains/crawler/README.md` | YouTube 获客、邮箱抽取、导入客户边界 | 改爬虫/导入前 |
| **Domain：Pricing** | `domains/pricing/README.md` | 价目表、阶梯报价、AI 匹配边界 | 改价目表/AI 报价前 |
| **Domain：Storage** | `domains/storage/README.md` | 双库、Migration、WAL 并发 |
| **Domain：Runtime/Worker** | `domains/runtime/README.md` | Worker 进程、background_job 队列 |
| **Domain：OCR** | `domains/ocr/README.md` | OCR 仅 Worker；主进程入队/读结果 |
| **Architecture** | `docs/architecture/database-schema.md` | 全表 DDL |
| **Architecture** | `docs/architecture/process-model.md` | UI 与 Worker 分工 |
| **Child Changes** | `changes/2026/07/chg-20260720-013-*.md` 等 | 单次可验收变更的目标、范围、验收 | 开始写代码前 |

## 4. 里程碑

| Milestone | 状态 | 目标 | Epic 子任务 | 验收 |
|-----------|------|------|-------------|------|
| **M1** 客户档案地基 | proposed | 客户模型、详情页、爬虫导入、**Playwright 邮箱补全** | CHG-013, CHG-014, **CHG-031** | S1 |
| **M2** 邮件模板 + 收发 | proposed | SMTP、模板、**IMAP 收信**、手动兜底 | CHG-015, CHG-026, **CHG-029** | S2, S2b, S2c, **S2d** |
| **M3** 价目表 + AI | proposed | 价目表、只读 Query、LLM 配置、邮件润色、**纠错记忆** | CHG-016–018, CHG-027, **CHG-030** | S4, S5, S10, **S12** |
| **M4** 报价/合作审计 | proposed | 正式合作字段 UI、报价变更史、时间线 | CHG-019 | S3 |
| **M5** WhatsApp 辅助 | proposed | Business API 桌面收发、翻译、建议、**webhook 部署手册** | CHG-020, **ADR-0004** | S6, **S6b** |
| **M6** Worker + OCR | proposed | Worker 骨架、**Tesseract**、语言包按需下载、识别管线、**商务场景** | CHG-023, CHG-025, CHG-024, **CHG-028** | S7, S8, S9, **S11** |

## 5. 依赖顺序

```text
M1（客户档案 + 爬虫导入 + **Playwright 邮箱补全 CHG-031**，依赖 CHG-023 Worker）
  → M2（SMTP 发信 + **IMAP 收信 CHG-029** + 手动兜底 CHG-026）
    → M3（价目表 + LLM 配置 + AI 只读 + 润色 + **纠错记忆 CHG-030**）
      → M4（合作/报价变更 UI + 历史审计）
        → M5（WhatsApp Business 辅助，绑同一客户）
          → M6（opendesk-worker + OCR，**不得阻塞 UI 主进程**）
```

**硬依赖说明：**

- M1 含 CHG-031（Playwright 邮箱补全），**实施依赖 CHG-023 Worker 骨架**；可与 CHG-013/014 并行开发，联调需 Worker 就绪
- M2 依赖 M1：发信必须关联 `customer_id`
- M3 依赖 M1 + M2：AI 需要客户档案与价目表；邮件起草需要 SMTP 已通
- M4 可与 M3 部分并行，但 AI 只读工具需在 M3 完成
- M5 依赖 M1 + M4：WA 会话必须绑客户；合作/报价字段需稳定

## 6. 跨领域改动总览

| 层级 | M1 | M2 | M3 | M4 | M5 |
|------|----|----|----|----|-----|
| `contracts/` | customer DTO | mail send IPC | pricing + agent query tools + mail draft | quote history events | channel message IPC |
| `crates/` | storage/customer, user? | mail, mail-net | agent query port, pricing | customer audit | channel |
| `python/` | — | — | agent readonly tools handler | — | translate/suggest |
| `apps/desktop/` | customer feature | mail feature | pricing UI, AI draft panel | customer edit forms | channel feature |
| `docs/managed/` | domains + changes | change | ADR + changes | change | change |

## 7. 架构不变量（全 MVP 遵守）

1. **React → Rust → Python**，Rust 是唯一协调者
2. **Contract First**：跨端变更顺序 Contract → Codegen → Rust → Python → React
3. **Python 不直连 SQLite**；AI 查客户/价目表走 Rust 只读 Query Port（见 ADR-0001）
4. **发送动作（邮件/WA）仅由人在 UI 触发**，AI 只产出草稿/建议
5. **客户写操作仅 UI/Rust UseCase**，带审计
6. **OCR 与重 CPU/IO 任务仅在 `opendesk-worker` 进程**；Tesseract + 本地 tessdata（ADR-0002、ADR-0003）
7. **tessdata 不随安装包分发**；用户在前端按需下载语言包
8. **`opendesk.db` 使用 WAL**；Worker 写、主进程短读

## 8. 叙事与架构文档

产品叙事以本路线图为准。历史「WhatsApp 自动客服」描述已废弃并改写，见 [CHG-021](changes/2026/07/chg-20260720-021-product-narrative-realignment.md)。

- 产品架构：[`docs/architecture/product-architecture.md`](../../architecture/product-architecture.md)
- 架构设计：[`database-schema.md`](../../architecture/database-schema.md)、[`process-model.md`](../../architecture/process-model.md)
- 团队评审：[`MVP_REVIEW.md`](../MVP_REVIEW.md)

## 9. 已完成摘要

（尚无；M1 完成后在此链接 Change 结果。）

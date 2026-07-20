---
id: CHG-20260720-024-ocr-worker-pipeline
title: Tesseract OCR 识别管线（仅 Worker 进程）
type: change
status: proposed
priority: P1
owner: developer
domain: ocr
parent: EPIC-20260720-001-mvp-sales-workbench
depends_on:
  - CHG-20260720-023-opendesk-db-worker-skeleton
  - CHG-20260720-025-ocr-tesseract-model-download
  - CHG-20260720-013-customer-profile-model
blocks: []
milestone: M6
created: 2026-07-20
updated: 2026-07-20
contracts: ocr IPC + DTO + events
related:
  - ADR-0002-heavy-work-worker-process
  - ADR-0003-tesseract-local-model-on-demand-download
---

# Tesseract OCR 识别管线（仅 Worker 进程）

## 目标

使用 **Tesseract** 对图片/PDF 做本地 OCR：**识别只在 `opendesk-worker`**；依赖 CHG-025 已安装的 `tessdata`；主进程入队/查结果；写入 `ocr_*` 表。

## 非目标

- 主进程内 Tesseract（**绝对禁止**）
- 安装包捆绑语言模型
- 云 OCR API
- Python Tesseract / sqlite3 读 OCR 表

## 背景

ADR-0003：引擎 Tesseract；语言包用户按需下载。本 Change 实现识别管线。

## 影响与边界

### 修改范围

| 层级 | 路径 | 改动 |
|------|------|------|
| Contract | `contracts/schema/v1/ocr/ipc/submit.request.schema.json` | path + `language_codes[]` + customer_id? |
| Contract | `contracts/schema/v1/ocr/ipc/submit.response.schema.json` | job_id / 错误码 OCR_LANGUAGE_NOT_INSTALLED |
| Contract | `contracts/schema/v1/ocr/ipc/get_result.*` | pages + blocks |
| Contract | `contracts/schema/v1/ocr/ipc/cancel.request.schema.json` | |
| Storage | Migration `create_ocr_tables` | ocr_job（含 language_codes）, ocr_document, ocr_page, ocr_text_block |
| Rust | `crates/ocr-engine/`（新建，**仅 worker 依赖**） | Tesseract Rust 绑定；`TESSDATA_PREFIX` |
| Rust | `crates/ocr/src/app/submit.rs` | 校验语言包 installed → 入队 |
| Rust | `crates/ocr/src/app/query.rs` | 只读结果 |
| Worker | `crates/worker/src/handlers/ocr.rs` | 解码 → Tesseract → 写 DB |
| Frontend | `apps/desktop/src/features/ocr/` | 提交（选语言）/ 进度 / 结果 |
| Customer | `customer_timeline` | ocr_completed |

### 不修改范围

| 路径 | 原因 |
|------|------|
| `crates/app` → `ocr-engine` | ADR-0002 |
| tessdata 下载逻辑 | CHG-025 |
| 安装脚本打包 traineddata | ADR-0003 禁止 |

### 风险

- PDF 分页内存：Worker 流式处理
- Tesseract 系统依赖：文档记录 Windows 运行时要求（或静态链接策略）

## 依赖关系

- 前置：CHG-023、**CHG-025**（须有已安装 tessdata）、CHG-013（可选 customer）
- ADR：0002、0003

## 实施方案

1. `ocr-engine`：封装 Tesseract，`set_tessdata_prefix({data}/OpenDesk/tessdata)`。
2. `ocr.submit`：校验 `language_codes` 均在 `ocr_language_pack.status=installed` → 入队。
3. Worker：claim → 分页 → `Tesseract::new(lang, OEM)` → `ocr_text_block`。
4. 进度 Event；UI 不阻塞。
5. 缺语言包时 UI 跳转语言包下载页。

## 验收

- [ ] 已安装 `eng` 后可 OCR 英文图片
- [ ] 未安装语言时 submit 返回明确错误
- [ ] UI 主进程无 OCR CPU 峰值
- [ ] `crates/app` 不链接 ocr-engine
- [ ] 安装包不含 `.traineddata`

## 实际结果

（完成前留空。）

## 后续项

- AI 对 OCR 文本摘要（Agent Change）

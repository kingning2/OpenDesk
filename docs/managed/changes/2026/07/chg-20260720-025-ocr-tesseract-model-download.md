---
id: CHG-20260720-025-ocr-tesseract-model-download
title: Tesseract 语言包目录与用户按需下载
type: change
status: proposed
priority: P1
owner: developer
domain: ocr
parent: EPIC-20260720-001-mvp-sales-workbench
depends_on:
  - CHG-20260720-023-opendesk-db-worker-skeleton
blocks:
  - CHG-20260720-024-ocr-worker-pipeline
milestone: M6
created: 2026-07-20
updated: 2026-07-20
contracts: ocr model IPC + events
related:
  - ADR-0003-tesseract-local-model-on-demand-download
---

# Tesseract 语言包目录与用户按需下载

## 目标

1. 定稿 **Tesseract 本地 tessdata** 存储与 DB 元数据；
2. 前端提供 **语言包列表 + 点击下载/删除/重试**；
3. **安装包不捆绑**任何 `traineddata`；
4. 主进程负责 HTTP 下载与进度；Worker 仅读取已安装文件。

## 非目标

- Tesseract 识别循环（CHG-024）
- 安装时静默下载或必选语言
- 云 OCR API
- Python 下载或识别

## 背景

用户确认：OCR 引擎为 **Tesseract**；模型 **本地**；由用户在 UI **自行选择下载**，不在安装阶段附带。

见 [ADR-0003](../../decisions/ocr/adr-0003-tesseract-local-model-on-demand-download.md)。

## 影响与边界

### 修改范围

| 层级 | 路径 | 改动 |
|------|------|------|
| Contract | `contracts/schema/v1/ocr/dto/language_pack.schema.json` | 语言包 DTO |
| Contract | `contracts/schema/v1/ocr/ipc/model_list.response.schema.json` | 可选+已安装状态 |
| Contract | `contracts/schema/v1/ocr/ipc/model_download.request.schema.json` | language_id |
| Contract | `contracts/schema/v1/ocr/ipc/model_download.response.schema.json` | 接受/拒绝 |
| Contract | `contracts/schema/v1/ocr/ipc/model_delete.request.schema.json` | 卸载 |
| Contract | `contracts/schema/v1/ocr/event/model.progress.schema.json` | bytes 进度 |
| Contract | `contracts/schema/v1/ocr/event/model.completed.schema.json` | |
| Storage | Migration `create_ocr_language_pack` | 表 + 种子（eng, chi_sim, chi_tra 等） |
| Rust | `crates/ocr/src/app/model_catalog.rs` | 列表 |
| Rust | `crates/ocr/src/app/model_download.rs` | 主进程 HTTP 下载到 tessdata 目录 |
| Rust | `crates/ocr/src/app/model_delete.rs` | 删文件 + 更新状态 |
| Rust | `crates/ocr/src/infra/tessdata_paths.rs` | `{data}/OpenDesk/tessdata/` |
| Frontend | `apps/desktop/src/features/ocr/ocr-language-packs-page.tsx` | 下载 UI |
| Frontend | `apps/desktop/src/features/ocr/use-ocr-language-pack.ts` | download/delete/progress |
| Frontend | `apps/desktop/src/features/setting/` 或 OCR 设置入口 | 导航至语言包页 |
| Docs | `database-schema.md` §5.5 `ocr_language_pack` | 同步 |

### 不修改范围

| 路径 | 原因 |
|------|------|
| `crates/ocr-engine/` Tesseract 绑定 | CHG-024 |
| 安装包/CI 打包 tessdata | ADR-0003 禁止 |
| Worker 内 HTTP 下载 | ADR-0003：下载在主进程 |

### Contract

- 需要 ocr model IPC + Event

### 跨层

- React → 主进程下载 → 写磁盘 + DB → Event → React 进度条

### 风险

- tessdata 镜像 URL 失效 → 配置化 `tessdata_base_url`（设置或内置默认 GitHub raw）
- 大文件下载中断 → 支持断点续传或失败后重试（MVP 可整文件重下）

## 依赖关系

- 前置：CHG-023（`opendesk.db` 基础）
- 阻塞：CHG-024（无语言包则 OCR 拒绝入队）
- ADR：ADR-0003

## 实施方案

1. Migration + 种子：`ocr_language_pack`（见 database-schema）。
2. 磁盘目录：`{app_data}/OpenDesk/tessdata/`；安装程序 **不创建** 预置 `.traineddata`。
3. `model_list`：返回种子语言 + `status`（not_installed / downloading / installed / failed）。
4. `model_download`：主进程 async HTTP → 写 `eng.traineddata` → 更新 `installed_at`；发 Event 进度。
5. UI：每语言一行，未安装显示「下载」；下载中进度条；已安装显示「删除」。
6. **禁止**应用启动时自动下载任何语言包。

### 种子语言（MVP 建议）

| id | display_name | 文件 |
|----|--------------|------|
| `eng` | English | `eng.traineddata` |
| `chi_sim` | 简体中文 | `chi_sim.traineddata` |
| `chi_tra` | 繁体中文 | `chi_tra.traineddata` |

URL 来源：tessdata 官方仓库或项目配置的 HTTPS 镜像（实施时写入种子 `download_url`）。

## 验收

- [ ] 全新安装后 `tessdata/` 目录无 `.traineddata`
- [ ] UI 可列出语言包且均为未安装
- [ ] 点击下载后可见进度并完成安装
- [ ] 安装后 DB `status=installed` 且文件存在
- [ ] 可删除已安装语言包
- [ ] 安装程序/打包脚本 **未**包含 traineddata
- [ ] 无启动时自动下载

## 实际结果

（完成前留空。）

## 后续项

- [CHG-024 OCR Worker 管线](chg-20260720-024-ocr-worker-pipeline.md)
- 可选：用户手动导入本地 `.traineddata` 文件

---
id: ADR-0003-tesseract-local-model-on-demand-download
title: OCR 使用 Tesseract 本地模型，用户按需下载
status: accepted
domain: ocr
created: 2026-07-20
supersedes: none
related:
  - ADR-0002-heavy-work-worker-process
---

# OCR 使用 Tesseract 本地模型，用户按需下载

## Context

OpenDesk OCR 需满足：

1. **引擎选型**：用户确认使用 **Tesseract**；
2. **本地优先**：识别在本地完成，不上传图像到云端 OCR API；
3. **安装包精简**：**不在安装时捆绑** `tessdata` 语言包（体积大、语言需求因人而异）；
4. **用户自选**：通过 **前端点击下载** 所需语言模型，可随时在设置/OCR 页管理；
5. **进程隔离**：识别在 Worker 进程（ADR-0002），下载与 UI 交互在主进程。

Tesseract 依赖 `*.traineddata` 文件（`TESSDATA_PREFIX` 目录）。无已安装语言包时不得静默失败，须明确提示用户下载。

## Decision

### 1. OCR 引擎

| 项 | 选择 |
|----|------|
| 引擎 | **Tesseract**（Rust 侧 `ocr-engine` crate 封装，**仅 `opendesk-worker` 链接**） |
| 推理位置 | Worker 进程 |
| 云端 OCR | **禁止**（MVP 及可预见版本） |

### 2. 语言模型（tessdata）

| 项 | 规则 |
|----|------|
| 存储路径 | `{data_local}/OpenDesk/tessdata/*.traineddata` |
| 安装包 | **不包含**任何 `traineddata` |
| 首次使用 | 用户打开 OCR/设置页 → 浏览语言列表 → **点击下载** |
| 下载执行 | **Tauri 主进程** HTTP 下载（async + 进度 Event）；**不在 Worker 内下载**（避免与 OCR 任务争用且便于 UI 展示） |
| 校验 | 可选 SHA256；失败标 `failed` 可重试 |
| 删除 | 用户可卸载已下载语言包（删除文件 + 更新 DB） |

### 3. 元数据

- `opendesk.db.ocr_language_pack` 表记录：语言代码、显示名、下载 URL、本地状态、`installed_at`。
- Migration **种子数据**写入可选语言目录（如 `eng`、`chi_sim`、`chi_tra`），默认均为 `not_installed`。
- `ocr.submit` 须指定或推断 `language_codes`；**任一未安装则拒绝入队**，返回可展示错误码 `OCR_LANGUAGE_NOT_INSTALLED`。

### 4. Worker 使用模型

```text
Worker 启动 OCR 任务前：
  读取 job.payload.language_codes
  校验 tessdata 目录下对应 .traineddata 存在
  设置 TESSDATA_PREFIX={data}/OpenDesk/tessdata
  调用 Tesseract（组合语言如 eng+chi_sim）
```

Worker **不发起**模型下载；仅读取已安装文件。

### 5. 前端职责

- **OCR 语言包**页面/面板：列表、已安装/未安装、下载进度、删除、重试。
- 下载按钮 → `ocr.model.download` IPC → 主进程下载 → Event `ocr.model.progress` / `completed`。
- 提交 OCR 前：若缺语言包，引导至下载页（不自动后台下载）。

### 6. 与安装程序关系

| 阶段 | 包含内容 |
|------|----------|
| `pnpm tauri build` / 安装包 | 应用、`opendesk-worker`、**无 tessdata** |
| 用户首次 OCR | 须先下载至少一种语言包 |

## Alternatives

| 方案 | 未选原因 |
|------|----------|
| 安装包内置常用语言 | 增大安装体积；用户要求自选 |
| 首次启动自动下载 | 未经用户同意占带宽/磁盘 |
| Python pytesseract | OCR 须在 Rust Worker；与 ADR-0002 一致 |
| 云 OCR API | 违背本地优先与隐私 |
| Worker 内下载模型 | 进度难展示；与 OCR 任务耦合 |

## Consequences

**正面：**

- 安装包小；用户按需选语言；
- 模型路径单一、可备份/迁移；
- 下载与识别职责分离清晰。

**成本：**

- 需维护语言包目录 URL（如 tessdata 官方/GitHub 镜像）及版本策略；
- 无网络时无法首次下载（可后续支持离线导入 `.traineddata` 文件）。

**兼容要求：**

- CHG-025（模型下载）、CHG-024（OCR 管线）须引用本 ADR；
- `ocr-engine` 不得被 `crates/app` 依赖（同 ADR-0002）。

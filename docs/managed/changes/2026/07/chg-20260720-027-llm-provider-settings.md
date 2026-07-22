---
id: CHG-20260720-027-llm-provider-settings
title: LLM Provider 配置 UI 与安全存储
type: change
status: completed
priority: P0
owner: developer
domain: agent
parent: EPIC-20260720-001-mvp-sales-workbench
depends_on: []
blocks:
  - CHG-20260720-017-ai-readonly-query-tools
  - CHG-20260720-018-ai-mail-draft
  - CHG-20260720-020-whatsapp-business-assist
milestone: M3
created: 2026-07-20
updated: 2026-07-22
contracts: runtime llm_settings IPC + DTO
related: []
---

# LLM Provider 配置 UI 与安全存储

## 目标

MVP **必须**提供桌面端 LLM 配置能力，使用户在启用 AI 功能（邮件润色、WA 翻译/建议、只读查库对话）前可配置模型 Provider。完成后：

- 设置页可配置：Provider 类型、API Base URL（可选）、API Key、默认模型 ID
- 凭据经 Rust `keyring` 存 OS 钥匙串；IPC get/save **永不**回传 api_key（含脱敏）
- Sidecar 启动或保存后由 Rust 注入 `OPENDESK_LLM_*` 环境变量（密钥仅内存，不落盘 Python）
- 未配置时测试连接返回明确错误码 `LLM_NOT_CONFIGURED`

## 非目标

- 多 Provider 负载均衡、自动 failover（MVP 后）
- 本地 Ollama/lmstudio 完整发现 UI（MVP 可支持 OpenAI 兼容 Base URL 填 localhost）
- 用量统计、计费仪表盘
- 在 React 内保存 API Key 到 localStorage
- AI 业务入口 CTA（由 CHG-018 等后续接 `LLM_NOT_CONFIGURED`）

## 背景

设置页原仅有语言与 YouTube API Key；Agent Ping 骨架无生产级 Provider 配置。M3 全部 AI 能力依赖 LLM。

## 影响与边界

### 修改范围

| 层级 | 路径 | 改动内容 |
|------|------|----------|
| Contract | `contracts/schema/v1/runtime/dto/llm_*.schema.json` | provider / settings DTO |
| Contract | `contracts/schema/v1/runtime/ipc/llm_*.schema.json` | get/save/test IPC；get **无** api_key |
| Contract | `contracts/schema/v1/runtime/sidecar/llm_test_connection.*` | Rust→Python 探测 |
| Rust | `crates/ports/src/llm_settings.rs` | Port + keyring 常量 |
| Rust | `crates/storage/src/llm_settings/` | SQLite 元数据 + `keyring` crate |
| Rust | `crates/app/src/commands/llm.rs` | `llm_settings_*` / `llm_test_connection` |
| Rust | `crates/runtime/.../lifecycle.rs` | `process_env` 注入 |
| Python | `python/packages/agent/src/agent/provider/` | config + factory |
| Python | `gateway/handlers/llm_test_connection.py` | Sidecar handler |
| Frontend | `features/setting/llm-*` + SettingsDialog | 预设 + 表单 + i18n |

### 不修改范围

| 路径 | 原因 |
|------|------|
| Python 直连读 OS keychain | 配置所有权在 Rust |
| 企业 SSO / 集中密钥管理 | MVP 后 |

### Contract

- runtime llm_settings schema（0.1.12）

### 跨层

- React → Rust（keyring）→ Sidecar env / probe payload → Python Provider

## 依赖关系

- 父任务：EPIC-20260720-001
- 前置任务：无
- 阻塞：CHG-017、CHG-018、CHG-020

## 实施方案

1. Contract：api_key **仅**出现在 save / test request 与 sidecar request；get/save response 仅 `has_api_key` / `configured`。
2. Rust：`keyring` crate（service=`OpenDesk`, user=`llm_api_key`）+ `opendesk.db.llm_setting` 元数据。
3. 设置页：厂商预设（OpenAI / Anthropic / DeepSeek / Ollama / Custom）+ 测试连接 + 保存。
4. 保存后重载 Sidecar，注入 `OPENDESK_LLM_*`。

## 验收

- [x] 设置页可保存 OpenAI 兼容 Provider（含 DeepSeek / Ollama 预设）
- [x] API Key 不出现在 get 响应、日志摘要、前端持久化；仅用 `has_api_key` 布尔
- [x] `llm_test_connection` 经 Sidecar `models.list` 轻量探测；未配置返回 `LLM_NOT_CONFIGURED`
- [x] `cargo check`（MSVC）ports/storage/runtime/adapter/app 通过；`storage` keyring 集成测试通过；Python handler smoke ok
- [x] 架构检查无新增 LLM 相关违规（全仓既有 unwrap 启发式失败与本 Change 无关）

## 实际结果

- 落地 CHG-027：Contract 0.1.12、keyring 存储、设置弹窗「AI / LLM」分区、Sidecar `/v1/runtime/llm_test_connection`。
- 密钥策略收紧：相对原稿「masked 回传」，改为 **完全不回传**，仅 `has_api_key`。
- 厂商预设：OpenAI / Anthropic / DeepSeek / 豆包 / Kimi / Ollama / Custom；Python `provider/vendors/` 分文件实现。
- Sidecar HTTP 接口收拢至 `python/sidecar/sidecar/api/`；platform 导出 `LLM_NOT_CONFIGURED` / `isLlmConfigured` 供后续 AI 入口复用。

## 后续项

- 多 Provider Profile / failover
- 本地 Ollama 一键发现模型列表
- AI 入口（邮件润色等）接 `LLM_NOT_CONFIGURED` CTA

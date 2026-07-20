---
id: CHG-20260720-027-llm-provider-settings
title: LLM Provider 配置 UI 与安全存储
type: change
status: proposed
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
updated: 2026-07-20
contracts: runtime llm_settings IPC + DTO
related: []
---

# LLM Provider 配置 UI 与安全存储

## 目标

MVP **必须**提供桌面端 LLM 配置能力，使用户在启用 AI 功能（邮件润色、WA 翻译/建议、只读查库对话）前可配置模型 Provider。完成后：

- 设置页可配置：Provider 类型、API Base URL（可选）、API Key、默认模型 ID
- 凭据经 Rust 安全存储（与 SMTP 同策略），不进日志与 Contract 明文
- Sidecar 启动或任务前由 Rust 注入 Provider 配置（环境变量或 IPC payload 引用，非前端直传 Python）
- 未配置时 AI 功能入口显示引导，调用返回明确错误码 `LLM_NOT_CONFIGURED`

## 非目标

- 多 Provider 负载均衡、自动 failover（MVP 后）
- 本地 Ollama/lmstudio 完整发现 UI（MVP 可支持 OpenAI 兼容 Base URL 填 localhost）
- 用量统计、计费仪表盘
- 在 React 内保存 API Key 到 localStorage

## 背景

当前设置页仅有语言与 YouTube API Key；Agent Ping 骨架无生产级 Provider 配置。M3 全部 AI 能力依赖 LLM，须在实施 CHG-017/018/020 前或同期交付本 Change。

## 影响与边界

### 修改范围

| 层级 | 路径 | 改动内容 |
|------|------|----------|
| Contract | `contracts/schema/v1/runtime/dto/llm_provider.schema.json` | provider 枚举 + model_id |
| Contract | `contracts/schema/v1/runtime/dto/llm_settings.schema.json` | 设置 DTO（无 api_key 明文） |
| Contract | `contracts/schema/v1/runtime/ipc/llm_settings_get.response.schema.json` | 脱敏展示（如 `sk-...xxxx`） |
| Contract | `contracts/schema/v1/runtime/ipc/llm_settings_save.request.schema.json` | 保存配置 |
| Contract | `contracts/schema/v1/runtime/ipc/llm_test_connection.response.schema.json` | 连通性探测 |
| Rust | `crates/runtime/src/llm_settings/`（新建） | 读写 + 安全存储 |
| Rust | `crates/agent/src/app/sidecar_env.rs` | 启动 Sidecar 时注入 Provider 配置 |
| Rust | `crates/app/` | 注册 runtime/llm_* IPC |
| Python | `python/packages/agent/agent/provider/config.py` | 从 Rust 注入读取配置 |
| Python | `python/packages/agent/agent/provider/factory.py` | OpenAI 兼容 + Anthropic（至少一种） |
| Frontend | `apps/desktop/src/features/setting/llm-settings-panel.tsx`（新建） | Provider 表单 |
| Frontend | `apps/desktop/src/features/setting/settings-page.tsx` | 嵌入 LLM 卡片 |
| Frontend | `apps/desktop/src/features/setting/use-llm-settings.ts` | IPC hook |
| Frontend | i18n `settings.llm*` 键 | 中英文文案 |
| Docs | `docs/managed/domains/agent/README.md` | LLM 配置说明 |

### 不修改范围

| 路径 | 原因 |
|------|------|
| Python 直连读 OS keychain | 配置所有权在 Rust |
| 企业 SSO / 集中密钥管理 | MVP 后 |

### Contract

- **需要** runtime llm_settings schema

### 跨层

- React → Rust 存密钥 → Rust 启动/调用 Sidecar 时注入 → Python Provider

### 风险

- Sidecar 热更新配置：MVP 保存后提示「重启 AI 服务」或 Rust 重载 Sidecar
- 测试连接消耗 token：使用最小 `models.list` 或轻量 ping

## 依赖关系

- 父任务：EPIC-20260720-001
- 前置任务：无（可与 M1/M2 并行）
- 阻塞：CHG-017、CHG-018、CHG-020

## 实施方案

1. Contract 定义 `llm_settings`；api_key 仅在 save request 出现，get 返回 masked。
2. Rust 使用 Tauri secure store / OS keychain 存 `api_key_ref`。
3. 设置页：Provider 下拉（`openai` / `anthropic` / `openai_compatible`）、Base URL、Model ID、API Key、「测试连接」「保存」。
4. Sidecar 启动参数或首包 handshake 携带 provider 配置（密钥由 Rust 内存传递，不落盘 Python）。
5. 各 AI 入口（邮件润色、WA 建议）在 `LLM_NOT_CONFIGURED` 时展示跳转设置的 CTA。

## 验收

- [ ] 设置页可保存 OpenAI 或 OpenAI 兼容 Provider 并成功测试连接
- [ ] API Key 不出现在日志、Contract get 响应明文、前端持久化存储
- [ ] 配置后 Agent Ping / 轻量 completion 成功
- [ ] 未配置时 AI 功能有明确阻断提示
- [ ] 架构检查通过

## 实际结果

（完成前留空。）

## 后续项

- 多 Provider 优先级与 fallback
- 本地 Ollama 一键发现

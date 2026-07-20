---
id: ADR-0004-whatsapp-webhook-deployment
title: WhatsApp Business Webhook 部署与开发接入
type: adr
status: accepted
domain: channel
created: 2026-07-20
updated: 2026-07-20
deciders: product-owner
related:
  - CHG-20260720-020-whatsapp-business-assist
  - EPIC-20260720-001-mvp-sales-workbench
---

# WhatsApp Business Webhook 部署与开发接入

## Status

Accepted — MVP 必须交付可操作的部署文档与最小 dev 方案。

## Context

WhatsApp Cloud API 要求 Meta 向 **公网 HTTPS** 回调 URL 推送入站消息。OpenDesk 是本地桌面应用，无内置公网 IP。CHG-020 实现 Rust webhook 处理，但若无部署说明，M5 无法验收。

约束：

- Webhook **必须经 Rust** 验签、幂等、入库（禁止 Meta → Python 直连）
- MVP 不要求企业级高可用集群，但须文档化 **开发 / 预发 / 生产** 三档选项
- 出站发送走 Rust → Business API，不依赖 webhook 反向通道

## Decision

### 1. Webhook 入口统一为 Rust HTTP 服务

| 组件 | 职责 |
|------|------|
| `crates/channel/src/adapter/webhook_server.rs` | 监听 `GET` 验证、`POST` 入站 |
| `crates/channel/src/app/inbound_webhook.rs` | `X-Hub-Signature-256` 验签、去重、写 DB、发 Tauri Event |
| Meta Developer Console | 配置 Callback URL + Verify Token |

桌面主进程在本地启动 webhook 监听（默认 `127.0.0.1:8787` 或配置项）；**公网暴露由外部 tunnel/relay 完成**。

### 2. 三档部署选项（MVP 文档 + CHG-020 验收须覆盖）

#### 开发（Developer）

```text
Meta → HTTPS tunnel（ngrok / cloudflared）→ localhost:8787 → Rust webhook
```

- 推荐 **cloudflared** 或 **ngrok** 免费隧道
- 仓库提供 `docs/architecture/whatsapp-webhook-deployment.md` 逐步说明
- 可选脚本：`scripts/dev/wa-tunnel.ps1`（仅文档化命令，不强制）

验收：开发机 + 隧道 + 测试号可收一条入站消息并在桌面 UI 显示。

#### 预发（Staging）

```text
Meta → 企业 VPS/Nginx（TLS）→ 转发 → 销售办公网内 relay 或单人桌面（固定出口）
```

- 使用子域名 `wa-hook.staging.example.com`
- Nginx `proxy_pass` 到内网 relay 或 VPN 可达的桌面端口
- **不推荐** 长期将隧道 URL 绑生产 WABA

#### 生产（Production）— MVP 最小方案

```text
Meta → 轻量 Relay 服务（VPS）→ WSS/长轮询 → 桌面 OpenDesk 客户端
```

MVP **实现路径 A**（优先）：

- 小型 **Relay**（Rust 或 Node，单二进制）：公网 HTTPS 收 webhook → 验签可选在 relay 或桌面（**最终验签必须在桌面 Rust 或 relay 与桌面共享 app secret 策略二选一，文档明确**）
- 桌面通过 **出站 WSS** 注册 `device_id`，relay 推送标准化入站事件
- 适合多销售各自桌面、无固定公网 IP

MVP **备选路径 B**（单人/小团队）：

- 固定一台 **always-on** 机器跑 OpenDesk + cloudflared 命名隧道
- 适合早期演示，文档标注运维成本

**MVP 不实现：** 多区域 HA、自动扩缩、K8s Ingress 模板（可列后续项）。

### 3. 安全基线

| 项 | 要求 |
|----|------|
| Verify Token | 环境变量或安全存储，不进仓库 |
| App Secret | 仅 Rust relay/webhook 层验签 |
| 幂等 | `wa_message_id` UNIQUE |
| TLS | 公网入口必须 HTTPS（隧道或 Nginx） |
| 日志 | 不记录完整消息体到普通日志（可配置 debug） |

### 4. 交付物（CHG-020 须包含）

- [`docs/architecture/whatsapp-webhook-deployment.md`](../../../architecture/whatsapp-webhook-deployment.md) — 操作手册
- Meta Console 配置截图说明（verify + subscribe `messages`）
- 开发隧道一键命令示例
- Relay 架构图（路径 A）

## Alternatives Considered

| 方案 | 不选原因 |
|------|----------|
| 仅文档不写 relay | 多桌面场景无法稳定收消息 |
| Meta → Python Sidecar | 违反 Rust 协调者边界 |
| 内置 OpenDesk 公网服务器 | 桌面安全与运维复杂 |
| 仅手动轮询 API 拉消息 | 非实时，API 限制多，体验差 |

## Consequences

### Positive

- M5 可验收「真实入站消息」
- 开发与生产路径清晰，团队可分工（infra vs 桌面）

### Negative

- 路径 A 需额外 relay 组件（可最小化单文件服务）
- 企业防火墙可能阻断 WSS，须文档化代理配置

## Compliance

- CHG-020 验收项增加：按 ADR-0004 完成 dev 隧道联调
- `domains/channel/README.md` 链接本 ADR 与部署手册

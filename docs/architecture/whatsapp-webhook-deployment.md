# WhatsApp Business Webhook 部署手册

> **已废弃（2026-07-21）** — WhatsApp 通道已改为 [ADR-0006](../managed/decisions/channel/adr-0006-whatsapp-baileys-worker.md) **Baileys Worker** 方案，不再使用 Business Cloud API + Webhook。下文仅作历史参考。

---

# WhatsApp Business Webhook 部署手册（历史）

本文曾对应已 superseded 的 [ADR-0004](../managed/decisions/channel/adr-0004-whatsapp-webhook-deployment.md) 与旧版 [CHG-020](../managed/changes/2026/07/chg-20260720-020-whatsapp-business-assist.md)。

## 1. 架构概览

```text
                    ┌─────────────────────────────────────┐
                    │     Meta WhatsApp Cloud API         │
                    └─────────────────┬───────────────────┘
                                      │ HTTPS webhook
                    ┌─────────────────▼───────────────────┐
  开发：ngrok/       │  公网 HTTPS 入口（隧道 / Relay / Nginx） │
  cloudflared       └─────────────────┬───────────────────┘
                                      │
                    ┌─────────────────▼───────────────────┐
                    │  OpenDesk Rust webhook handler        │
                    │  GET  /webhook/whatsapp  (verify)   │
                    │  POST /webhook/whatsapp  (inbound)  │
                    └─────────────────┬───────────────────┘
                                      │
                    ┌─────────────────▼───────────────────┐
                    │  SQLite + Tauri Event → React UI    │
                    └─────────────────────────────────────┘
```

**硬规则：** Meta 回调只到 Rust；禁止 Meta → Python 直连。

## 2. 前置条件

| 项 | 说明 |
|----|------|
| Meta 开发者账号 | [developers.facebook.com](https://developers.facebook.com) |
| WhatsApp Business 测试号 | Cloud API 测试号码 |
| App Secret | 用于 `X-Hub-Signature-256` 验签 |
| Verify Token | 自定义字符串，与 OpenDesk 配置一致 |
| OpenDesk | CHG-020 已构建，本地 webhook 端口可配置（默认 `8787`） |

## 3. 开发环境（推荐首选验收）

### 3.1 启动 OpenDesk webhook

确保桌面端已启动，Rust 监听 `http://127.0.0.1:8787/webhook/whatsapp`（路径以 Contract 为准）。

### 3.2 启动 HTTPS 隧道

**cloudflared（示例）：**

```powershell
cloudflared tunnel --url http://127.0.0.1:8787
```

记录输出的 `https://*.trycloudflare.com` 地址。

**ngrok（示例）：**

```powershell
ngrok http 8787
```

### 3.3 配置 Meta Developer Console

1. 应用 → WhatsApp → Configuration
2. **Callback URL：** `https://<tunnel-host>/webhook/whatsapp`
3. **Verify Token：** 与 OpenDesk 设置中 `WA_VERIFY_TOKEN` 一致
4. 点击 **Verify and Save**
5. Subscribe 字段：`messages`（至少）

### 3.4 验证

1. 用测试号向 Business 号码发一条消息
2. 桌面 Channel 页应出现入站消息
3. `customer_timeline` 写入 `wa_in`（绑 customer 后）

### 3.5 常见问题

| 现象 | 排查 |
|------|------|
| Verify 失败 | Token 不一致；隧道未指向 8787 |
| 收不到 POST | 未 subscribe `messages`；隧道断开 |
| 验签失败 | App Secret 配置错误 |

## 4. 预发环境（Staging）

适用于团队共享测试号、固定子域名。

```text
Meta → https://wa-hook.staging.example.com/webhook/whatsapp
     → Nginx (TLS)
     → 内网 OpenDesk 或 staging relay
```

Nginx 示例（仅示意）：

```nginx
location /webhook/whatsapp {
    proxy_pass http://127.0.0.1:8787;
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-Proto $scheme;
}
```

须使用有效 TLS 证书（Let's Encrypt 等）。

## 5. 生产最小方案

### 路径 A：Relay + 桌面 WSS（多销售桌面）

```text
Meta → Relay (VPS, HTTPS) → WSS push → 各 OpenDesk 客户端
```

- Relay 公网收 webhook，按 `phone_number_id` / `device_id` 路由
- 桌面启动时 WSS 注册，保持长连接
- 验签可在 Relay（持有 App Secret）或端到端加密 payload（实施时二选一，须安全评审）

适合：销售各自笔记本，无公网 IP。

### 路径 B：Always-on 单机 + 命名隧道（小团队演示）

- 一台常开 PC 跑 OpenDesk + cloudflared 命名隧道
- 运维简单，不适合大规模

**MVP 验收：** 至少完成 **§3 开发隧道**；路径 A/B 写入运维 runbook，Relay 可与 CHG-020 分阶段实现。

## 6. 安全清单

- [ ] App Secret / Verify Token 不进 git
- [ ] 公网入口仅 HTTPS
- [ ] `wa_message_id` 幂等去重
- [ ] 生产日志脱敏
- [ ] 出站发送仅 `sent_by=human` UI 触发

## 7. 相关文档

- [ADR-0004](../managed/decisions/channel/adr-0004-whatsapp-webhook-deployment.md)
- [Channel Domain](../managed/domains/channel/README.md)
- [CHG-020](../managed/changes/2026/07/chg-20260720-020-whatsapp-business-assist.md)

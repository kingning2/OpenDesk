---
id: ADR-0008-browser-login-hybrid
title: 闲鱼登录载体换浏览器快照 + 混合收发 + 内嵌 Webview
type: adr
status: accepted
domain: channel
created: 2026-08-11
updated: 2026-08-11
deciders: kingning2
supersedes: none
related:
  - CHG-20260811-002-channel-browser-login
  - CHG-20260811-001-channel-xianyu-customer-service
  - ADR-0007-channel-protocol-dispatcher
---

# 闲鱼登录载体换浏览器快照 + 混合收发 + 内嵌 Webview

> Playwright 放 Python 是 [ADR-0009](../python-runtime/adr-0009-python-only-when-rust-insufficient.md) 的典型例外（Rust 无成熟等价库）。自动回复链路中的「Python LLM」已由 ADR-0009 取代为 Rust 默认。

## Status

Accepted。

## Context

现有闲鱼通道用 **Cookie 字符串**（DevTools 复制）登录，走 WebSocket 协议直连收发。缺点：

1. 单一 cookie 字符串不含浏览器指纹/localStorage/请求头，容易被风控判定为自动化。
2. 用户希望换成 [ai-goofish-monitor](https://github.com/Usagi-org/ai-goofish-monitor) 的登录载体 —— **Chrome 扩展导出的完整浏览器快照**（cookies + env 指纹 + localStorage + 请求头），这套方案 14k star 验证过、抗风控更好。
3. 用户希望在桌面应用内**看到闲鱼页面本身**（不只是消息流）。

同时确认了三个技术事实：

- **ai-goofish-monitor 没有任何客服/发消息能力**（纯找货监控），grep 全源码零 IM 代码 —— 消息收发无法依赖它，必须保留现有 WS 协议。
- **Tauri 2.11.5 支持内嵌外部站点**：`WebviewWindowBuilder::new(app, label, WebviewUrl::External(url))` + `WebviewWindow::set_cookie`（走 WebView2 cookie manager），CSP 为 null 不挡外部导航。
- Playwright 在 Python sidecar 会触及「Python 不直连渠道协议」硬约束。

## Decision

### 1. 登录载体：浏览器快照

- 账号凭据 `credential` 升级为存**完整快照 JSON**（Chrome 扩展导出：cookies[] + env + storage + headers），字段类型保持 `string` 不 breaking。
- Python sidecar 用 **Playwright** 恢复会话：`storage_state={"cookies": snapshot["cookies"]}` + `_build_context_overrides`（还原 UA/locale/timezone/viewport/指纹）+ `_build_extra_headers`（附加白名单请求头）+ `add_init_script` 剥离 webdriver 指纹。
- 登录成功后 **导出登录后的 cookies 数组**，回传给 Rust。

### 2. 消息收发：混合模式（浏览器登录 + WS 协议收发）

- Playwright 只负责登录、保持会话、导出 cookies。
- Rust 拿到 cookies 数组后仍走**现有 WebSocket 协议直连**收发（保留已验证的 `xianyu/{api,message,ws}.rs`）。
- 自动回复链路：**Rust 调度 → 默认 Rust LLM → Rust 安全过滤 → 发送**（Python LLM 仅 ADR-0009 例外）。

### 3. 页面呈现：Tauri 内嵌 Webview 显示闲鱼

- `tauri::WebviewWindowBuilder` 加载 `https://www.goofish.com`，用 `WebviewWindow::set_cookie` 注入快照 cookies。
- 用户可在桌面应用内看到并操作真实闲鱼页面。

### 4. 架构约束调整（Python 与渠道协议）

Playwright 放 Python sidecar **调整**「Python 不直连渠道协议」约束为：

> Python 可做**登录与浏览器会话管理**（Playwright 生态缺口），但**不做业务调度、不做默认 LLM 决策**；消息收发仍由 Rust 经协议完成。

**Why**：Playwright 是 Python/Node 生态成熟方案（Rust 无等价库），登录载体切换必须依赖它；业务调度与 LLM 决策默认全在 Rust（ADR-0009）。

## Alternatives

- **Playwright 放 Node 子进程**：满足「Python 不碰渠道」，但需新增运行时，与现有 sidecar 重复 → 不选。
- **Rust 原生浏览器**：无成熟库，工作量大 → 不选。
- **纯浏览器收发**（Playwright 全程，弃 WS 协议）：需从零开发闲鱼 IM 自动化（页面依赖强、易失效），且 ai-goofish-monitor 无现成代码 → 不选，改为混合。
- **不显示页面只显示消息流**：用户明确要看到闲鱼页面 → 不选。

## Consequences

- **正面**：登录抗风控更强；桌面内可看/操作真实页面；WS 收发机制复用、工作量可控。
- **成本**：Playwright 依赖 + chromium 浏览器二进制（打包体积增大）；WebView2 指纹与 Chrome 不同，闲鱼可能风控（需 on_navigation + UA 注入缓解）；侧边约束调整需文档同步。
- **文档**：`domains/channel/README.md`、`CHG-20260811-002` 需同步修订。
- **兼容**：`credential` 字段类型不变，旧 cookie 字符串仍可读（兼容）。

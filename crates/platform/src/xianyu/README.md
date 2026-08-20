# 闲鱼平台协议（`crates/platform/src/xianyu`）

Rust 侧闲鱼 HTTP / mtop / WebSocket 实现。对上层暴露 `XianyuChannel`、`MtopClient`、`fetch_user_profile`、`fetch_seller_items` 等。

## 新增 mtop 接口前必读

**接口发现流程、抓包步骤、已接入 api 登记表：**

[`skills/dingda/guides/xianyu-mtop-discovery.md`](../../../../skills/dingda/guides/xianyu-mtop-discovery.md)

## 模块一览

| 文件 | 职责 |
|------|------|
| `mtop.rs` | 通用 mtop 客户端（签名、set-cookie、重试） |
| `profile.rs` | 用户资料 `mtop.idle.web.user.page.nav` |
| `item.rs` | 卖家商品列表 `mtop.idle.web.xyh.item.list` |
| `session.rs` | IM 会话同步 `mtop.taobao.idlemessage.pc.session.sync` |
| `api.rs` | WS token、`hasLogin` |
| `ws.rs` | WebSocket 消息通道 |
| `cookie.rs` / `http.rs` / `sign.rs` | Cookie 解析、HTTP 客户端、签名 |

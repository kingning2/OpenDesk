# 闲鱼平台协议（`crates/platform/src/xianyu`）

Rust 侧闲鱼 HTTP / mtop / WebSocket 实现。目录对齐 [goofish-cli](https://github.com/fancyboi999/goofish-cli)：`core/` + 领域模块。

对上层仍通过 `mod.rs` re-export：`XianyuChannel`、`MtopClient`、`fetch_user_profile`、`fetch_seller_items` 等。

## 新增 mtop 接口前必读

**接口发现流程、抓包步骤、已接入 api 登记表：**

[`skills/dingda/guides/xianyu-mtop-discovery.md`](../../../../skills/dingda/guides/xianyu-mtop-discovery.md)

## 模块一览

| 路径 | 职责 |
|------|------|
| `core/mtop.rs` | 通用 mtop 客户端（签名、set-cookie、重试） |
| `core/api.rs` | WS token、`hasLogin` |
| `core/ws.rs` | WebSocket 消息通道（含开发帧隧道附着） |
| `core/dev_tunnel.rs` | 开发态帧隧道协议 |
| `core/session.rs` | IM 会话同步 `mtop.taobao.idlemessage.pc.session.sync` |
| `core/cookie.rs` / `cookies.rs` / `http.rs` / `sign.rs` | Cookie、HTTP、签名 |
| `message/frames.rs` | 出站帧（/reg、heartbeat、listUserMessages） |
| `message/history.rs` | 历史消息解析 |
| `item/` | 卖家商品列表 |
| `profile/` | 用户资料与会话商品卡 |

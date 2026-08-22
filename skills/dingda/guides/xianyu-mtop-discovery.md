# 闲鱼 mtop 接口发现与接入

> 作者：Xiaoman  
> 创建时间：2026-08-20  
> 适用：`crates/platform/src/xianyu/src/` 新增平台 HTTP 能力时

闲鱼 Web 端**没有面向第三方的公开 REST API**。业务数据通过阿里 **mtop** 协议下发（`h5api.m.goofish.com`），需登录 Cookie + MD5 签名。本指南记录**如何发现新接口**、**如何登记**、**如何在 Rust 接入**。

---

## 〇、open.goofish.com 开放平台能不能用？

**结论：DingDa 桌面客服当前场景（扫码登录个人账号 → 同步商品 / 收发消息）不能靠它替代 mtop；两者是不同体系。**

| 维度 | [open.goofish.com](https://open.goofish.com)（闲鱼**小程序**开放平台） | 当前 DingDa 方案（Web mtop + WS） |
|------|------------------------------------------------------------------------|-----------------------------------|
| **面向谁** | 受邀 **ISV 服务商**，开发跑在闲鱼 App 内的小程序 H5 | 桌面端，用户用**个人闲鱼账号**扫码登录 |
| **准入** | 企业淘宝账号 + 运营邀请；[文档写明不对外公开申请](https://open.goofish.com/doc/quick-start.html) | 无需开放平台入驻 |
| **鉴权** | AppKey/Secret + OAuth `accessToken`；服务端需部署 **聚石塔** | 账号 **Cookie** + mtop 签名 |
| **[openChat](https://open.goofish.com/doc/api/advanced/openChat.html)** | **Windvane 前端 JS API**：在小程序容器里**唤起聊天 UI** | 不适用；我们在 Rust 里走 WebSocket 协议收发包 |
| **商品列表** | **无**「查卖家全部上架商品」接口；订单创单需事先找运营定位**单个** `item_id` | `mtop.idle.web.xyh.item.list` 拉卖家主页在售列表 |
| **典型 TOP 接口** | `alibaba.idle.isv.order.query`、发货、退款、用户信息等 | 抓包 mtop（订单/详情等待补） |

**何时考虑开放平台？** 若未来 DingDa 以 **ISV 小程序**形态嵌入闲鱼 App、走官方订单/支付链路，再读 [服务端接入文档](https://open.goofish.com/doc/development/dev/server.html) 申请 TOP 权限。这与「桌面客服 + 个人账号 Cookie」是两条产品线，**不能混用鉴权**。

---

## 一、标准发现流程（以后新接口都走这套）

### 1. 浏览器抓包

1. 用 **已登录** 的 Chrome / Edge 打开 [https://www.goofish.com](https://www.goofish.com)（与桌面扫码登录同一账号）。
2. `F12` → **Network** → 筛选 **Fetch/XHR**。
3. 在过滤框输入 `mtop` 或 `h5api.m.goofish.com`。
4. **复现目标操作**（例如：打开「我的发布」、进入卖家主页、点开订单、发消息等）。
5. 找到新出现的请求，记录以下字段：

| 记录项 | 在哪里看 |
|--------|----------|
| **api** | Query 参数 `api=`，如 `mtop.idle.web.xyh.item.list` |
| **version** | Query 参数 `v=`，通常 `1.0` |
| **data** | Request Payload → `data`（JSON 字符串） |
| **响应结构** | Response → `data` 节点；`ret[0]` 含 `SUCCESS` 为成功 |
| **触发页面 / 操作** | 便于后人复现，如「卖家主页下滑加载更多」 |

### 2. 确认登录态依赖

- 请求 Header 必须带 **Cookie**（含 `unb`、`_m_h5_tk` 等）。
- 签名 token 取自 `_m_h5_tk` 的 `_` 前缀；过期时响应 set-cookie 下发新 token，需重签重试（已在 [`MtopClient`](../../../../crates/platform/src/xianyu/src/core/mtop.rs) 实现）。

### 3. Rust 接入（固定模式）

```
抓包确认 api + data
  → crates/platform/src/xianyu/src/<能力>.rs  声明 MtopRequest + 解析响应
  → Tauri ipc/<模块>.rs                   编排（读账号 Cookie → 调 platform → 写 business store）
  → 登记本文「已接入接口表」
```

调用示例：

```rust
let client = MtopClient::new(cookie_str)?;
let request = MtopRequest::new(
    "mtop.idle.web.xyh.item.list",
    "1.0",
    serde_json::json!({
        "userId": user_id,
        "pageNumber": 1,
        "scene": "seller_home",
    }),
);
let response = client.call(&request).await?;
```

公共常量：[`crates/common/src/constants.rs`](../../../../crates/common/src/constants.rs) → `xianyu::H5_API_BASE`、`WEB_ORIGIN`。

### 4. 登记与验证

- 在本文件 **「已接入接口表」** 增加一行（api、版本、用途、发现来源、Rust 文件）。
- 在对应 `*.rs` 模块头注释写上 api 名，并链回本指南。
- 本地用真实 Cookie 点一次桌面功能验证；失败时回浏览器对比 `data` / `ret` 是否变化。

---

## 二、商品列表接口（CHG-20260820-001 本次来源）

### 发现来源

| 类型 | 说明 |
|------|------|
| **社区逆向文章** | [闲鱼爬虫爬爬乐：抓包分析与 Playwright 实战](https://www.sakasa.cn/posts/paxianyu/) — 明确写出接口名 `mtop.idle.web.xyh.item.list` 与 `data` 字段 |
| **浏览器复现** | 打开卖家主页 `https://www.goofish.com/personal?userId={userId}`，向下滑动加载更多；Network 中出现 `item.list` 请求 |

> 说明：阿里**未**对普通开发者开放「我的发布列表」Open API；淘宝开放平台 [`alibaba.idle.item.user.publishitems`](https://open.alitrip.com/docs/api.htm?apiId=56245) 面向 ISV 授权场景，**不适用于**当前桌面 Cookie 登录方案。

### 接口规格（已验证可接入）

| 项 | 值 |
|----|-----|
| **api** | `mtop.idle.web.xyh.item.list` |
| **version** | `1.0` |
| **method** | POST（mtop 表单，`data` 为紧凑 JSON） |
| **请求 data** | `{ "userId": "<卖家ID>", "pageNumber": <页码从1起>, "scene": "seller_home" }` |
| **userId** | 当前实现取账号 Cookie 的 `unb`（[`XianyuAccount::extract_unb`](../../../../business/src/account/mod.rs)） |
| **列表节点** | `data.cardList` 或 `data.items` |
| **itemId** | `cardData.detailParams.itemId` 或 `data.itemId` |
| **标题** | `cardData.main.title` 等 |
| **价格** | `cardData.main.soldPrice` / `soldPrice`（字符串或数字） |
| **翻页** | `pageNumber` 递增，空页或重复 id 则停止 |

### 代码落点

| 层 | 路径 |
|----|------|
| 平台 API | [`crates/platform/src/xianyu/src/item/mod.rs`](../../../../crates/platform/src/xianyu/src/item/mod.rs) → `fetch_seller_items` |
| IPC | [`apps/desktop/src-tauri/src/platforms/xianyu/ipc/item.rs`](../../../../apps/desktop/src-tauri/src/platforms/xianyu/ipc/item.rs) → `item_sync` |
| 前端 | [`apps/desktop/src/features/xianyu/items.tsx`](../../../../apps/desktop/src/features/xianyu/items.tsx) →「同步商品」 |

---

## 三、已接入接口表（持续维护）

新接口接入后**必须**追加一行。

| api | v | 用途 | 发现方式 | Rust 实现 | 备注 |
|-----|---|------|----------|-----------|------|
| `mtop.idle.web.user.page.nav` | 1.0 | 登录用户昵称 / 头像 | 连接后抓包 / 社区资料 | [`profile.rs`](../../../../crates/platform/src/xianyu/src/profile/profile.rs) | `data={}` |
| `mtop.idle.web.xyh.item.list` | 1.0 | 卖家主页在售商品列表 | [sakasa 文章](https://www.sakasa.cn/posts/paxianyu/) + 卖家主页抓包 | [`item.rs`](../../../../crates/platform/src/xianyu/src/item/mod.rs) | 商品同步 |
| `mtop.taobao.idlemessage.pc.session.sync` | 3.0 | IM 会话列表基线（仅活跃 Top N） | [goofish-cli list-chats](https://github.com/fancyboi999/goofish-cli) | [`session.rs`](../../../../crates/platform/src/xianyu/src/core/session.rs) | **完整会话列表走 WS `userConvs` 推送**（`ackDiff pts=0` 请求全量，参考 goofish-cli `collect_session_cids`）；本接口只是基线，会漏掉非活跃会话 |
| `mtop.taobao.idlemessage.pc.login.token` | 1.0 | WebSocket 注册 token | IM 连接抓包 | [`api.rs`](../../../../crates/platform/src/xianyu/src/core/api.rs) | 非 MtopClient，独立实现 |
| `/r/MessageManager/listUserMessages`（WS LWP） | — | 拉取会话完整消息历史 | [goofish-cli message history](https://github.com/fancyboi999/goofish-cli) `commands/message/history.py` / `core/ws.py` | [`ws.rs`](../../../../crates/platform/src/xianyu/src/core/ws.rs) `fetch_user_messages` | WebSocket LWP 请求/响应：mid 关联、`userMessageModels` 分页、内容 `content.custom.data` base64→JSON 解码 |
| `mtop.taobao.idle.pc.detail` | 1.0 | 商品详情（文案 / 高清图） | [sakasa 文章](https://www.sakasa.cn/posts/paxianyu/) + Goofish Client 文档 | [`item.rs`](../../../../crates/platform/src/xianyu/src/item/mod.rs) `fetch_item_detail` | 响应含 `shareInfoJsonString` 需二次 JSON 解析 |
| `mtop.idle.trade.pc.message.headinfo` | 1.0 | 会话关联商品卡信息（标题/图/价格） | 网页端聊天抓包 | [`headinfo.rs`](../../../../crates/platform/src/xianyu/src/headinfo.rs) `fetch_message_headinfo` | **GET** 请求；`data={"itemId":...,"sessionId":...,"sessionType":1}`（`MtopRequest::with_get()`） |
| `mtop.idle.web.trade.rate.list` | — | 买家评价列表 | 业务注释 | **未实现** | 见 `business/src/delivery/data.rs` 注释 |

> **WS 会话同步（非 HTTP）**：侧栏完整会话列表由 `wss://wss-goofish.dingtalk.com/` 推送。注册后发 `/r/SyncStatus/ackDiff` 且 **`pts=0`** 请求全量同步（用当前时间戳只会推连接后的新消息），服务器随后推 `body.userConvs[]`（每项含 `cid`、`extension.extUserId`/`itemId`/`itemTitle`、`visible`、`modifyTime`）。实现见 [`ws.rs`](../../../../crates/platform/src/xianyu/src/core/ws.rs) `sync_user_convs`；会话存储新增 `cid`（goofish 会话 id），消息历史/发送/商品卡都以 `cid` 为会话标识。

---

## 四、待接入时优先抓包的操作

| 业务 | 建议操作 | 预期 api 关键词 |
|------|----------|-----------------|
| 我的发布 / 在售管理 | 个人中心 → 我发布的 | `item` · `publish` · `sell` |
| 订单列表 | 我买到的 / 我卖出的 | `trade` · `order` |
| 发货 | 订单详情点发货 | `delivery` · `ship` |
| 商品详情 | 点开任意商品页 | `detail`（已有 `mtop.taobao.idle.pc.detail` 线索） |
| 发布商品 | 发布页提交 | `publish`（社区提及 `mtop.idle.pc.idleitem.publish`） |

---

## 五、常见问题

**Q：同步 0 条商品？**  
对比浏览器同账号卖家主页是否有货；检查 `userId` 是否与抓包一致（有时需数字 id 而非 `unb` 格式，以抓包为准）。

**Q：ret 含 TOKEN_EXPIRED？**  
`MtopClient` 会自动写回 set-cookie 并重试；仍失败则重新扫码登录。

**Q：新接口放 Python 还是 Rust？**  
按 ADR-0009：**默认 Rust**（`crates/platform`）；仅 Playwright 等 Rust 不够时才走 Python Sidecar。

---

## 六、相关文件

- mtop 客户端：[`crates/platform/src/xianyu/src/core/mtop.rs`](../../../../crates/platform/src/xianyu/src/core/mtop.rs)
- 模块索引：[`crates/platform/src/xianyu/src/README.md`](../../../../crates/platform/src/xianyu/src/README.md)
- 变更记录：[`docs/managed/changes/2026/08/chg-20260820-001-xianyu-item-sync.md`](../../../../docs/managed/changes/2026/08/chg-20260820-001-xianyu-item-sync.md)

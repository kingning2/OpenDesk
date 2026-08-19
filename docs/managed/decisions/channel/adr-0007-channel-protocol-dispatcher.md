---
id: ADR-0007-channel-protocol-dispatcher
title: 多渠道通道层：ChannelProtocol trait + Dispatcher
type: adr
status: accepted
domain: channel
created: 2026-08-11
updated: 2026-08-11
deciders: kingning2
supersedes: none
related:
  - CHG-20260811-001-channel-xianyu-customer-service
---

# 多渠道通道层：ChannelProtocol trait + Dispatcher

> **部分替代：** 「Python 仅大模型接入」已由 [ADR-0009](../python-runtime/adr-0009-python-only-when-rust-insufficient.md) 取代。LLM 默认在 Rust；Python 只补生态缺口。渠道 trait / Dispatcher / 调度在 Rust 的结论仍有效。

## Status

Accepted。

## Context

DingDa 是「React → Rust → Python」三层架构的 AI 客服桌面应用。产品需要**多渠道客服**（闲鱼、微信、WhatsApp、淘宝等），第一个落地闲鱼。需要回答三个问题：

1. **渠道代码放哪、如何分类**：`crates/` 被定位为「通用基建」，不承载单一渠道业务。
2. **业务调度归属**：谁来决定「收到消息要不要回、回什么、怎么发」。
3. **AI 自动发信**：仓库既有限制「AI 不自动发信」，而智能客服的核心诉求是自动回复。

## Decision

### 1. 渠道业务位置与分类

- `crates/` 仅保留通用基建（kernel 事件总线、common 契约、ports trait、storage 通用 SQLite、runtime sidecar 生命周期）。
- **真实渠道业务全部放 `apps/desktop/src-tauri/src/channels/`**，在该目录内按文件夹分类：
  - `protocol.rs` — 渠道统一 `ChannelProtocol` / `InboundListener` trait
  - `dispatcher.rs` — `ChannelDispatcher` 调度器（注册表 + 多账号生命周期 + 入站管线）
  - `xianyu/` — 闲鱼协议实现（第一个平台）
  - 未来 `wechat/`、`whatsapp/`、`taobao/` 加同构子目录，实现同一 trait

```rust
#[async_trait]
pub trait ChannelProtocol: Send + Sync {
    fn kind(&self) -> ChannelKind;
    async fn connect(&self, account: &ChannelAccount) -> Result<(), ChannelError>;
    async fn disconnect(&self) -> Result<(), ChannelError>;
    async fn send(&self, peer_id: &str, text: &str) -> Result<String, ChannelError>;
    fn connection_state(&self) -> ConnectionState;
    fn set_inbound_listener(&self, listener: Box<dyn InboundListener>);
}
```

**Why**：单一 trait 让新平台接入成本降到「实现协议细节」，调度/存储/UI 全部复用。

### 2. 业务调度在 Rust，LLM 默认也在 Rust

- Rust 是唯一协调者：收消息 → 归一化 → 入库 → **判断是否回复**（规则 + 上下文）→ **默认在 Rust 生成回复** → 安全过滤 → 发送 → 出站入库。
- 仅当 Rust 生态缺少可用实现时，才把该步骤放到 Python sidecar（ADR-0009）。不得把「大模型」预设为 Python 职责。
- 若走 sidecar，接口须渠道无关，且不含业务/渠道逻辑。

```text
入站消息 → Rust(回复决策/意图路由/上下文/默认 LLM) →（仅 ADR-0009 例外）Python → Rust(安全过滤) → 发送
```

**Why**：符合「Rust 唯一协调者」；意图路由与默认 LLM 在 Rust 可单测、可离线。Python 不直连渠道协议。

### 3. 全局自动回复（放宽「AI 不自动发信」）

- 默认开启**全局自动回复**，设置项可关闭（`channel_settings.auto_reply`）。
- 关闭或 LLM 失败时：**不自动发送**，仅把生成的回复作为「建议」展示在 UI 供人工一键发出。
- 安全过滤（屏蔽 微信/QQ/支付宝/银行卡/线下）在 Rust 侧强制执行，作为最后一道闸门。

**Why**：智能客服的核心诉求是无人值守自动回复，与既有「仅 UI 人工操作」约束冲突；本次以「默认开 + 可关闭 + 安全过滤兜底」作为产品级放宽，需要产品确认。

## Alternatives

- **每个渠道一个独立 crate**：与「crates 只放基建」冲突，渠道间重复代码无法收敛 → 不选。
- **Python 直连渠道协议**（照搬 XianyuAutoAgent 结构）：违反「Python 不直连渠道」硬约束，且多平台会散落在 Python → 不选。
- **人工确认制发送**（项目现状 WhatsApp 采用）：无法满足无人值守客服诉求，作为 `auto_reply=false` 的降级路径保留。

## Consequences

- **正面**：多渠道扩展成本低；业务调度与默认 LLM 集中在 Rust 可测；Python 只在生态缺口出现。
- **成本**：闲鱼协议为逆向非官方，接口可能失效需维护；自动回复需要合规与安全过滤保障。
- **文档**：`domains/channel/README.md`、`CHG-20260811-001` 需同步修订。
- **兼容**：沿用 `channel_*` 表命名约定；协议层消息 id 作为幂等键。

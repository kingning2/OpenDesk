---
id: ADR-0009-python-only-when-rust-insufficient
title: Python 仅在 Rust 生态不够时编写
type: adr
status: accepted
domain: python-runtime
created: 2026-08-19
updated: 2026-08-19
deciders: product-owner
supersedes:
  - ADR-0002 中「Python Sidecar：仅 AI 推理」
  - ADR-0007 中「Python 仅大模型接入」
related:
  - CHG-20260819-001-python-rust-first
  - ADR-0008-browser-login-hybrid
---

# Python 仅在 Rust 生态不够时编写

## Status

Accepted。

## Context

仓库长期把 Python Sidecar 写成「AI Runtime」：LLM、RAG、Agent 默认放 Python。这会把本可在 Rust 完成的能力（HTTP 调模型、编排、工具调用）固定到第二运行时。

Rust 是唯一协调者，也是默认实现语言。Python 进程有独立发行、类型与边界成本。只有 Rust 生态缺少可用实现时，才值得引入 Python。

ADR-0008 已按此原则把 Playwright 放 Python（Rust 无成熟等价库）。需要把该原则提升为全局决策，替代「Python = AI 层」。

## Decision

1. **默认用 Rust。** 业务编排、存储、渠道协议、Agent、LLM HTTP 调用、只读 Query Port 均在 Rust 实现。
2. **Python 是例外 Sidecar，不是 AI Runtime。** 仅当 Change Record 写明「Rust 生态缺少可用实现」时才新增或扩展 `python/**`。典型缺口如 Playwright 一类浏览器自动化。
3. **即便走 Python，边界不变：** React 不直连 Python；Python 不直连 SQLite、不写业务状态、不自动发信；Rust 管理 Sidecar 生命周期并转发事件。
4. **现有 PingAgent / sidecar 骨架可保留**，不作为「AI 必须在 Python」的依据。新能力不得默认加 Python handler。

## Alternatives

- **继续把 Python 当 AI Runtime：** 双运行时成为默认路径，Rust 生态能做的事也被拆走 → 不选。
- **删除 Python 目录：** 仍有真实生态缺口（如 Playwright），需要例外通道 → 不选。
- **每个缺口一个新运行时（Node 等）：** 与现有 sidecar 重复 → 不选。

## Consequences

- **正面：** AI 与业务默认留在 Rust，可单测、可离线编排；Python 体积与职责收敛。
- **成本：** 新增 Python 代码前必须论证生态缺口；旧文档中「Python 承担推理」不再作为现状。
- **兼容：** 不强制立刻把已有 Python 骨架迁回 Rust；新功能按本 ADR 选型。

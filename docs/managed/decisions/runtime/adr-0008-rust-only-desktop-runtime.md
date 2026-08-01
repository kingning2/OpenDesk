---
id: ADR-0008-rust-only-desktop-runtime
title: 桌面端采用 Rust 单运行时
status: accepted
domain: runtime
created: 2026-08-01
supersedes: Python Sidecar runtime
---

# 桌面端采用 Rust 单运行时

## Context

Python Sidecar 只负责 LLM HTTP 转发和少量文本处理，却要求额外维护子进程生命周期、HTTP 桥接、三端生成物、PyInstaller 产物和跨平台发布步骤。Rust 已具备直接完成这些工作的网络、存储和异步运行时能力。

## Decision

桌面端固定采用 `React → Tauri IPC → Rust`：

- React 不访问本地服务或第三方 AI API，只调用 Tauri IPC。
- Rust 是唯一应用运行时和协调者，直接访问 LLM、IMAP、SMTP、YouTube、SQLite 与系统 keyring。
- AI Provider 在 Rust 中按 OpenAI-compatible 与 Anthropic 两种协议实现；厂商差异通过配置和 Strategy 处理。
- Tauri shell 是 Composition Root；Feature 用例不读取全局状态。
- 跨 Feature 只通过 Contract、Event 或明确的 Query Port 协作。
- Contract codegen 只生成 Rust 与 TypeScript。

## Alternatives

- 保留 Python Sidecar：继续承担进程、打包和跨语言维护成本，拒绝。
- 将 AI 拆成独立 Rust Sidecar：当前没有隔离或独立扩缩容需求，属于额外进程复杂度，拒绝。
- 为每个 LLM 厂商使用独立 SDK：增加依赖和重复实现；现有两种 HTTP 协议足以覆盖，拒绝。

## Consequences

- 正面影响：减少运行进程、发布产物、跨语言契约和故障面；AI 与业务用例可在同一类型系统内组合。
- 成本与限制：Rust 需要自行维护两种 LLM HTTP 协议和错误映射。
- 后续兼容要求：新增 Provider 优先复用现有协议；只有协议确实不同且已有需求时才新增 Strategy。

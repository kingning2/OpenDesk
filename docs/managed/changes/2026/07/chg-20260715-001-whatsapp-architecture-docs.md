---
id: CHG-20260715-001-whatsapp-architecture-docs
title: 整合 WhatsApp 产品与 Python AI Runtime 架构文档
type: change
status: completed
priority: P1
owner: cursor-agent
domain: documentation
parent: none
depends_on: []
blocks: []
milestone: architecture-skeleton
created: 2026-07-15
updated: 2026-07-15
contracts: none
related: []
---

# 整合 WhatsApp 产品与 Python AI Runtime 架构文档

## 目标

在仓库根目录新增两份结构清晰、与 OpenDesk 现有三层架构一致的产品与 Python AI Runtime 架构文档。

## 非目标

- 不实现 WhatsApp、LLM、RAG、Tool Calling 或业务流程。
- 不新增或修改 Contract、代码、配置和依赖。
- 不改变当前 Architecture Skeleton 阶段。

## 背景

用户提供了两份原始 Markdown 内容，要求整合后保存为根目录文档。原始消息流包含 WhatsApp API 直达 Python 的描述，需要按 Rust 唯一协调者约束修正。

## 影响与边界

- 修改范围：根目录两份架构文档、当前 Change Record 和 Active Registry。
- 不修改范围：业务代码、契约、配置、依赖、既有稳定领域文档。
- Contract：无。
- 跨层：仅描述 React → Rust → Python 的既有边界。
- 跨 Feature：否。
- 风险：文档可能被误解为当前已实现能力，因此必须明确区分目标架构、MVP 与当前状态。

## 依赖关系

- 父任务：无。
- 前置任务：无。
- 阻塞任务：无。

## 实施方案

1. 将产品定位、业务流程、数据、安全和扩展方向整理为 `PRODUCT_ARCHITECTURE.md`。
2. 将 Python 模块边界、工程结构、处理流程、错误处理和阶段目标整理为 `PYTHON_AI_RUNTIME_ARCHITECTURE.md`。
3. 对齐 Rust 唯一协调者、Contract First、Python 禁止直接访问本地文件与 SQLite 的硬约束。
4. 标明文档描述的是目标架构，不代表相关业务能力已实现。

## 验收

- [x] 两份文档位于仓库根目录且 Markdown 结构完整。
- [x] WhatsApp、文件、SQLite 调用链不绕过 Rust。
- [x] Python Runtime 边界与现有 workspace 结构一致。
- [x] 当前能力与目标能力清晰区分。
- [x] 实际结果已回填。

## 实际结果

- 新增 `PRODUCT_ARCHITECTURE.md`，整合产品定位、部署边界、三端职责、业务流程、数据、安全和交付阶段。
- 新增 `PYTHON_AI_RUNTIME_ARCHITECTURE.md`，按现有 uv workspace 整理 sidecar、gateway、provider、agent、RAG、Prompt、Memory 和 Tool Calling 边界。
- 修正原始方案中 WhatsApp API 直达 Python 的调用链，统一由 Rust Channel Adapter 协调。
- 补充 WhatsApp webhook 需要公网入口、云模型数据出站和“本地优先”不等于数据绝不离机等现实约束。
- 验证：编辑器 Markdown 诊断无错误；`git diff --check` 通过。

## 后续项

- 具体业务实现需另建任务，并遵循 Contract → Codegen → Rust → Python → React。

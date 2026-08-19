# DingDa 项目说明

## 项目是什么

DingDa 是 **本地优先的 AI Agent 智能客服** 桌面应用：AI 提供客服场景的回复建议与交互。核心能力由 Rust 唯一协调者编排并默认实现（含 LLM）。Python Sidecar 只在 Rust 生态缺少可用实现时使用，不是 AI Runtime。

> 当前处于 **Architecture Skeleton + 基础切片** 阶段。UI Shell、Rust Agent 组装、Python Sidecar ping 已有。

## 文档入口

**请从这里开始：**

👉 [`docs/managed/`](managed/) — 领域文档、Change Record、ADR。

## 技术架构（不变）

```
React  →  Rust（默认实现，含 AI）
            ↓ 仅当 Rust 生态不够
         Python Sidecar
```

- 契约驱动：`contracts/` 为跨端唯一真相源
- Feature 隔离：跨 Feature 仅 Query Port · Event · Contract
- Python 例外原则：[ADR-0009](managed/decisions/python-runtime/adr-0009-python-only-when-rust-insufficient.md)
- 完整约束： [`.cursor/rules/master.md`](../.cursor/rules/master.md)

## 仓库结构

| 路径 | 说明 |
|------|------|
| `apps/desktop` | Tauri + React |
| `crates` | Rust Workspace（仅基建） |
| `python` | 例外 Sidecar（非 AI Runtime） |
| `contracts` | 跨端共享契约 |
| `docs/managed/` | **领域文档与变更协议** |
| `docs/architecture/` | 架构文档与 ADR |

## 开发

```bash
pnpm install
pnpm tauri dev
pnpm lint
python skills/dingda/scripts/check_architecture.py
```

## 其他文档

- [`README.md`](../README.md) — 工程入口
- [`docs/architecture/`](architecture/) — 架构文档与 ADR
- [`skills/dingda/`](../skills/dingda/) — AI 开发知识库

---

*本文替代原「统一 AI 智能客服平台」描述；旧叙事已废弃，见 [CHG-021](managed/changes/2026/07/chg-20260720-021-product-narrative-realignment.md)。*

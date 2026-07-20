# OpenDesk 项目说明

## 项目是什么

OpenDesk 是 **本地优先的 AI 商务桌面**：从 YouTube 获取潜客邮箱，用 **自有 SMTP** 邮件谈价，WhatsApp Business 在桌面辅助翻译与回复建议（**人工发送**）。系统记住每位客户的来源、报价与合作状态；AI 通过 **只读查询** 获取上下文，**不能改库、不能自动发信**。

> 当前处于 **Architecture Skeleton + 部分切片** 阶段。YouTube 爬虫、UI Shell、Sidecar 等已有；客户档案、邮件发信、价目表、AI 商务能力等 **按 MVP 规划待实现**。

## 详细规划文档（团队评审入口）

**请从这里开始：**

👉 [`docs/managed/MVP_REVIEW.md`](managed/MVP_REVIEW.md)

包含：路线图、Epic、8 个子任务 Change Record（含精确改动范围）、ADR、领域边界。

## 技术架构（不变）

```
React  →  Rust  →  Python
         ↑
    唯一协调者
```

- 契约驱动：`contracts/` 为跨端唯一真相源
- Feature 隔离：跨 Feature 仅 Query Port · Event · Contract
- 完整约束： [`.cursor/rules/master.md`](../.cursor/rules/master.md)

## 仓库结构

| 路径 | 说明 |
|------|------|
| `apps/desktop` | Tauri + React |
| `crates` | Rust Workspace |
| `python` | AI Runtime（Sidecar） |
| `contracts` | 三端共享契约 |
| `docs/managed/` | **MVP 规划与变更协议** |
| `docs/architecture/` | 产品与运行时架构 |

## 开发

```bash
pnpm install
pnpm tauri dev
pnpm lint
python skills/opendesk/scripts/check_architecture.py
```

## 其他文档

- [`README.md`](../README.md) — 工程入口
- [`docs/architecture/product-architecture.md`](architecture/product-architecture.md) — 产品架构
- [`skills/opendesk/`](../skills/opendesk/) — AI 开发知识库

---

*本文替代原「统一 AI 智能客服平台」描述；旧叙事已废弃，见 [CHG-021](managed/changes/2026/07/chg-20260720-021-product-narrative-realignment.md)。*

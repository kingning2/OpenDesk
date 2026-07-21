---
id: ROADMAP-email-agent-port-branches
title: email-agent 迁入 — 分支命令手册
type: roadmap
status: active
domain: product
updated: 2026-07-21
epic: EPIC-20260721-001-email-agent-port
---

# email-agent 迁入 — 分支命令手册

命名规范：`<role>/<kind>/<slug>`。创建后 **必须** 运行 `pnpm branch:sync` 刷新 [`.cursor/rules/active-branch.mdc`](../../../.cursor/rules/active-branch.mdc)。

Epic：[EPIC-20260721-001](../changes/2026/07/epic-20260721-001-email-agent-port.md)

---

## 0. 文档批次（当前）

仅改 `docs/**`，无业务代码：

```bash
pnpm branch:create main docs email-agent-port-docs
pnpm branch:sync
```

合并目标：`main`

---

## Phase 0 — 契约与地基

### 0a. Managed 文档（若与代码分批）

```bash
pnpm branch:create main docs email-agent-phase0-docs
pnpm branch:sync
```

### 0b. Contract schema（跨端契约）

```bash
pnpm branch:create contract chore email-agent-phase0-contracts
pnpm branch:sync
# 改 contracts/ 后：
pnpm sync:contracts   # 或项目约定的 codegen 命令
```

### 0c. KOL 预留（trait 桩 + 占位 UI，无对接逻辑）

```bash
pnpm branch:create frontend chore kol-port-placeholder
pnpm branch:sync
```

允许路径：`crates/ports/**`、`apps/desktop/**`（Coming Soon 页）、`contracts/schema/v1/kol/_reserved/**`

### 0d. ADR / Domain 随文档批已含在 0a；若单独评审：

```bash
pnpm branch:create main docs adr-0006-channel
pnpm branch:sync
```

---

## Phase 1 — 客户 + 建联阶段 + 工作流

扩展 CHG-013；通常跨 Contract + Rust + React：

```bash
# 1. 契约先行
pnpm branch:create contract feature customer-outreach-stage
pnpm branch:sync

# 2. Rust 存储 + customer/workflow crate
pnpm branch:create frontend feature customer-outreach-workflow
pnpm branch:sync
```

> `frontend` 角色含 `crates/**`；workflow 若在独立 crate 仍用 `frontend` 分支。

---

## Phase 2 — 邮件核心（IMAP/SMTP + 收件箱）

扩展 CHG-015 / CHG-026 / CHG-029：

```bash
pnpm branch:create contract feature mail-inbox-sync
pnpm branch:sync

pnpm branch:create frontend feature mail-net-imap-inbox
pnpm branch:sync
```

Worker IMAP job 与 `crates/worker` 同分支（`frontend` 含 crates）。

---

## Phase 3 — AI 草稿 + 人审发送 + 待发队列

扩展 CHG-018 / CHG-030：

```bash
pnpm branch:create contract feature mail-draft-pending-queue
pnpm branch:sync

pnpm branch:create frontend feature mail-batch-human-send
pnpm branch:sync

pnpm branch:create python feature agent-mail-wa-draft
pnpm branch:sync
```

---

## Phase 4 — 定价 + YT 定价引擎

扩展 CHG-016：

```bash
pnpm branch:create contract feature pricing-yt-engine
pnpm branch:sync

pnpm branch:create frontend feature pricing-yt-engine
pnpm branch:sync
```

---

## Phase 5 — WhatsApp Baileys Worker

扩展 CHG-020；ADR-0006：

```bash
# spike 可先用独立分支
pnpm branch:create frontend feature wa-baileys-spike
pnpm branch:sync

pnpm branch:create contract feature channel-baileys-ipc
pnpm branch:sync

pnpm branch:create frontend feature channel-baileys-worker
pnpm branch:sync
```

---

## Phase 6 — 统计 / 排期 / 数据迁移

```bash
pnpm branch:create contract feature analytics-schedule
pnpm branch:sync

pnpm branch:create frontend feature analytics-schedule-ui
pnpm branch:sync

pnpm branch:create main chore email-agent-data-migration
pnpm branch:sync
```

数据迁移 CLI 若纯 Rust 脚本，用 `frontend`；若独立工具且触达多域，用 `main`。

---

## Phase 7 — KOL 系统迁入（未来，独立 Epic）

```bash
pnpm branch:create contract feature kol-platform-port
pnpm branch:sync

pnpm branch:create frontend feature kol-channel-ui
pnpm branch:sync
```

---

## 依赖顺序（分支合并建议）

```text
main/docs/email-agent-port-docs
  → contract/*/phase0-contracts
    → frontend/*/customer-outreach-workflow
      → contract/*/mail-inbox-sync → frontend/*/mail-net-imap-inbox
        → python + frontend（Phase 3 AI）
          → frontend/*/pricing-yt-engine
            → frontend/*/channel-baileys-worker
              → frontend/*/analytics-schedule-ui
```

每分支合并 `main` 前：对应 Change Record `completed`、从 `ACTIVE.md` 移除。

---

## 常用命令

```bash
pnpm branch              # 交互式创建
pnpm branch:sync         # 切换分支后同步 Cursor 规则
pnpm lint                # 合并前全量检查
```

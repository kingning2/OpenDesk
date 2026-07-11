---
id: CHG-20260711-005-merge-main-conflicts
title: 解决 main 合并规则冲突
status: completed
owner: codex
domain: documentation
created: 2026-07-11
updated: 2026-07-11
---

# 快速修复

## 问题与目标

合并 main 时 `AGENTS.md` 与 `.cursor/rules/python.md` 发生冲突。目标是保留 main 的自动分支工作流，同时保留当前分支的 Managed Docs 门禁和完整 Python Runtime 约束。

## 边界

- 修改：仅两个冲突规则文件、当前 Change Record 和 Active 索引；
- 不修改：其他已正常合并的 main 内容及业务代码；
- Contract/跨层影响：无。

## 验收

- [x] 无未合并路径或冲突标记
- [x] main 分支自动 scope 规则保留
- [x] Managed Docs 门禁保留
- [x] Python分层、契约、命名和日志规则保留
- [x] 合并提交成功且工作区干净

## 结果

已将 `AGENTS.md` 合并为 main 的自动分支 scope + Managed Docs 门禁；将 Python 规则合并为 Sidecar 边界、运行时分层、命名、Contract和结构化日志的完整约束。冲突标记与 Markdown 差异检查通过。

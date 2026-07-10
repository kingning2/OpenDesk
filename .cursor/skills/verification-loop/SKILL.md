---
name: verification-loop
description: OpenDesk 项目最小验证闭环：lint + architecture checks + staged guardrail。
---

# Verification Loop (OpenDesk)

在以下时机执行：
- 完成一个可提交改动后
- 提交 PR 前
- 触及跨层/跨 feature 代码后

最小闭环：
1. `pnpm lint`
2. `python skills/opendesk/scripts/check_architecture.py`
3. `python skills/opendesk/scripts/check_boundary.py`
4. `python skills/opendesk/scripts/check_contracts.py`

如果任一步失败：
- 停止提交
- 优先修复边界/契约问题，再修复风格问题

通过标准：
- lint 全绿
- architecture / boundary / contracts 校验全绿
- 无绕过规则的临时改动（尤其是 lint/config 文件）

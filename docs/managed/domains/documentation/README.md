# Documentation Domain

## 职责

管理 `docs/managed/` 的 Domain、Roadmap、Change、ADR、Registry、模板与上下文预算。

- Domain 记录稳定当前事实。
- Roadmap 记录近期目标与顺序。
- Change 记录一次独立改动的意图、范围与结果。
- ADR 记录长期技术选择。
- Registry 只做导航。

## 边界

- 不替代 Contract、代码或测试。
- 不保存完整日志、生成物或逐文件 diff。
- 历史 Change/ADR 正文只读；当前事实变化写 Domain 或新的 Change。
- Managed Docs 门禁由根 `AGENTS.md` 规定，当前仍是 Agent 指令级约束。

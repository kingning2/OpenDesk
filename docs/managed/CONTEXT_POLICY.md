# AI 上下文读取策略

## 渐进读取

1. 固定入口：`docs/managed/README.md`、`registry/ACTIVE.md`。
2. 按路径只读一个 Domain；路由见 `registry/DOMAINS.md`。
3. 只读当前 Change Record；复杂 Child Change 可读父 Epic 摘要。
4. 仅在兼容性、设计冲突或历史原因不明时读取相关 ADR/历史 Change。

默认不递归读取 `docs/managed/`，不读无关已完成 Change 或被替代 ADR。

## 路由

- `contracts/**` → Contracts
- `crates/agent/**`、Agent/AI 基建与相关命令 → Agent
- `crates/worker/**`、后台任务 → Runtime / Worker
- 其他路径按 `registry/DOMAINS.md`

## 写入

- Change 记录本次意图与结果；Domain 记录稳定当前事实；ADR 记录长期选择。
- 不把日志、生成物或逐文件 diff 写入 Markdown。
- 历史 Change/ADR 正文只读；新事实写到当前 Change 或 Domain。
- 同一规则只在主入口完整描述，子文档链接引用。

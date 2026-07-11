# AI 上下文读取策略

## 目标

让 Agent 以最小上下文完成正确修改，避免因为文档增长而反复读取无关历史。

## 渐进式读取

### L0：固定入口

每次任务只读：

- `docs/managed/README.md`
- `docs/managed/registry/ACTIVE.md`

### L1：领域入口

根据修改路径映射到一个主领域，只读该领域的 `README.md`。领域入口应控制在 150 行以内。

### L2：当前变更

只读当前 Change Record，以及它明确列出的最多三个直接依赖文档。Child Change 可额外读取父 Epic 的摘要和子任务表，不读取其他无关 Child Change。

### L3：按需深入

仅在出现设计冲突、兼容性问题或历史原因不明时，读取相关 ADR 或历史 Change。

## 路由原则

- `python/**` → `domains/python-runtime/`
- `contracts/**` → `domains/contracts/`
- Agent 能力 → `domains/agent/`
- 无匹配领域时先创建领域入口，不向全局 README 塞正文。

后续新增领域时，将路径映射写入 `registry/DOMAINS.md`。

## 上下文预算

- 单次任务默认最多加载 5 份 managed docs。
- 复杂任务默认读取路径为：入口 → Active → Domain → 当前 Change → 父 Epic 摘要。
- 默认不读已完成 Change Record。
- 默认不读被替代 ADR。
- 引用只链接到直接依赖，不建立长链式“相关阅读”。
- 如果一份文档需要读取多个附录才能理解，应重新划分领域边界。

## 写入预算

- 每次只更新当前 Change Record、一个领域现状文件和必要的 ADR。
- Child Change 只更新父 Epic 中对应的一行状态，不向 Epic 复制实施正文。
- 高频进度不逐条追加；只记录阶段结果。
- 运行日志、测试完整输出和生成物不写入 Markdown，只记录命令、结论与外部路径。

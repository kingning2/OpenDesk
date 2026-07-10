---
name: search-first
description: 在写实现前先检索仓库内已有能力与约束，避免重复造轮子和跨层改动。
---

# Search First (OpenDesk)

适用场景：
- 新增 feature / crate / contract 前
- 想新增依赖、脚本、工具链前
- 不确定能力应该落在哪一层（React / Rust / Python）时

执行步骤：
1. 先确认边界：React -> Rust -> Python，禁止 React 直连 Python。
2. 先查仓库是否已有模板与脚手架：`skills/opendesk/templates/`、`skills/opendesk/scripts/`。
3. 先查契约是否已存在：`contracts/`。
4. 只有在“现有能力无法复用”时，才新增文件或模块。

输出要求：
- 说明复用了哪些现有结构
- 说明为什么不能复用（如需新增）
- 说明是否涉及跨端变更顺序：Contract -> Rust -> Python -> React

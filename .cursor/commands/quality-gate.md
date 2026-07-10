---
description: 运行 OpenDesk 最小质量门禁（lint + 架构边界校验）。
---

执行以下命令并汇总结果：

```bash
pnpm lint
python skills/opendesk/scripts/check_architecture.py
python skills/opendesk/scripts/check_boundary.py
python skills/opendesk/scripts/check_contracts.py
```

输出格式：
1. 每条命令是否通过
2. 若失败，列出首个关键错误
3. 给出“可提交 / 不可提交”结论

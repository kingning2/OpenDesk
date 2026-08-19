# Domains

Domain 文档描述某个领域的当前事实，不记录逐次开发过程。

每个领域目录至少包含一个 `README.md`，内容限制为：

- 职责与非职责；
- 稳定边界；
- 关键接口入口；
- 当前约束；
- 有效 ADR 链接。

领域入口接近 150 行时，按稳定子领域拆分；入口只保留导航和摘要。

## 索引

| 领域 | 文档 | 状态 |
|------|------|------|
| Workflow | [workflow/README.md](workflow/README.md) | planned |
| Channel (WA) | [channel/README.md](channel/README.md) | planned（Baileys，ADR-0006） |
| Pricing | [pricing/README.md](pricing/README.md) | planned |
| Analytics | [analytics/README.md](analytics/README.md) | planned |
| Schedule | [schedule/README.md](schedule/README.md) | planned |
| Alert | [alert/README.md](alert/README.md) | planned |
| KOL | [kol/README.md](kol/README.md) | **planned（暂缓对接）** |
| Agent | [agent/README.md](agent/README.md) | 骨架 |
| Python Sidecar（例外） | [python-runtime/README.md](python-runtime/README.md) | 骨架；非 AI Runtime |
| Runtime/Worker | [runtime/README.md](runtime/README.md) | planned |
| Storage | [storage/README.md](storage/README.md) | planned |
| OCR | [ocr/README.md](ocr/README.md) | 语言包下载已落地；识别未实现 |
| Contracts | [contracts/README.md](contracts/README.md) | — |

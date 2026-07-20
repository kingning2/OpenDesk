# Domain Registry

该表只负责把代码路径路由到领域入口。

| 领域 | 主要路径 | 入口 | 状态 |
|---|---|---|---|
| Product (MVP) | 跨域 | [mvp-sales-workbench roadmap](../roadmaps/mvp-sales-workbench.md) | active |
| Runtime / Worker | `crates/worker/**`, `crates/runtime/**` | [runtime](../domains/runtime/README.md) | planned |
| Storage | `crates/storage/**` | [storage](../domains/storage/README.md) | planned |
| OCR | `crates/ocr/**`, `crates/ocr-engine/**`（规划） | [ocr](../domains/ocr/README.md) | planned |
| Customer | `crates/customer/**`, `features/customer/**` | [customer](../domains/customer/README.md) | planned |
| Mail | `crates/mail/**`, `crates/mail-net/**`, `features/mail/**` | [mail](../domains/mail/README.md) | planned |
| Crawler | `crates/crawler/**`, `features/crawler/**` | [crawler](../domains/crawler/README.md) | active |
| Pricing | `crates/pricing/**`, `features/pricing/**` | [pricing](../domains/pricing/README.md) | planned |
| Channel | `crates/channel/**`, `features/channel/**` | [channel](../domains/channel/README.md) | planned |
| Python Runtime | `python/**` | [python-runtime](../domains/python-runtime/README.md) | active |
| Contracts | `contracts/**` | [contracts](../domains/contracts/README.md) | active |
| Agent | `python/packages/agent/**`, `crates/agent/**` | [agent](../domains/agent/README.md) | active |
| Documentation | `docs/managed/**` | [documentation](../domains/documentation/README.md) | active |

新增领域时一并定义互斥的管理范围；同一路径若匹配多个领域，以更具体的路径为主领域。

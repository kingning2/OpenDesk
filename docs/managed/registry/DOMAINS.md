# Domain Registry

该表只负责把代码路径路由到领域入口。

| 领域 | 主要路径 | 入口 | 状态 |
|---|---|---|---|
| Runtime / Worker | `crates/runtime/**` | [runtime](../domains/runtime/README.md) | planned |
| Storage | `crates/storage/**` | [storage](../domains/storage/README.md) | planned |
| OCR | —（规划中） | [ocr](../domains/ocr/README.md) | planned |
| Pricing | —（规划中） | [pricing](../domains/pricing/README.md) | planned |
| Channel | —（规划中） | [channel](../domains/channel/README.md) | planned |
| Python Runtime | `python/**` | [python-runtime](../domains/python-runtime/README.md) | active |
| Contracts | `contracts/**` | [contracts](../domains/contracts/README.md) | active |
| Agent | `apps/desktop/src-tauri/src/**` | [agent](../domains/agent/README.md) | active |
| Documentation | `docs/managed/**` | [documentation](../domains/documentation/README.md) | active |

新增领域时一并定义互斥的管理范围；同一路径若匹配多个领域，以更具体的路径为主领域。

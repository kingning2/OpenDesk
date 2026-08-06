# Domain Registry

| 领域 | 主要路径 | 入口 | 状态 |
|---|---|---|---|
| Product | 跨域 | [MVP roadmap](../roadmaps/mvp-sales-workbench.md) | active |
| Runtime / Worker | `crates/worker/**`, `crates/workflow_runtime/**`, `crates/kernel/**` | [runtime](../domains/runtime/README.md) | active |
| Storage | `crates/storage/**` | [storage](../domains/storage/README.md) | active |
| OCR | OCR Contract、Worker handler | [ocr](../domains/ocr/README.md) | planned |
| Customer | `crates/customer/**`, `features/customer/**` | [customer](../domains/customer/README.md) | active |
| Mail | `crates/mail/**`, `crates/mail-net/**`, `features/mail/**` | [mail](../domains/mail/README.md) | active |
| Crawler | `crates/crawler/**`, `crates/crawler-enrich/**`, `features/crawler/**` | [crawler](../domains/crawler/README.md) | active |
| Workflow Runtime | `crates/workflow_runtime/**` | [workflow-runtime](../domains/workflow-runtime/README.md) | active |
| Contracts | `contracts/**`, generated Rust/TS | [contracts](../domains/contracts/README.md) | active |
| Agent / LLM | `crates/agent/**`, Agent commands/features | [agent](../domains/agent/README.md) | active |
| Documentation | `docs/managed/**` | [documentation](../domains/documentation/README.md) | active |
| Knowledge | `crates/knowledge/**`, `features/knowledge/**` | [knowledge](../domains/knowledge/README.md) | active |

其他已登记领域入口保留在 `domains/`。同一路径命中多个领域时，以更具体的业务领域为主。

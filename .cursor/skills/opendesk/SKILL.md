---
name: opendesk
description: OpenDesk enterprise AI customer-service desktop platform development knowledge base. Enforces React→Rust→Python architecture, contracts-first, hexagonal design, feature boundaries, and skeleton-phase constraints. Use when working on OpenDesk, adding features/crates/contracts/IPC/events, architecture review, or any cross-layer change in this repository.
---

# OpenDesk Development Skill

## Quick Start

1. Read [skills/opendesk/README.md](../../skills/opendesk/README.md) for architecture and AI rules.
2. Identify task type → open matching **recipe** under `skills/opendesk/recipes/`.
3. Copy from **templates/** — never invent structure from scratch.
4. Run validation: `python skills/opendesk/scripts/check_architecture.py`
5. Run lint: `python skills/opendesk/scripts/lint_all.py`

## Hard Constraints (Skeleton Phase)

| Allow | Forbid |
|-------|--------|
| dirs, crates, traits, DTO, Contract, Interface, Mock | business logic, demos, architecture bypass |

## Architecture

```
React → Rust Application Core → Python Runtime
```

Rust is the **only coordinator**. React never knows Python. Python never knows React.

Cross-end change order: **Contract → Codegen → Rust → Python → React**

## When to Read What

| Task | Document |
|------|----------|
| Architecture overview | `architecture/overview.md` |
| Add feature | `recipes/add-feature.md` + `templates/feature/` |
| Add crate | `recipes/add-crate.md` + `scripts/create_crate.py` |
| Add Python package | `recipes/add-python-package.md` + `scripts/create_python_package.py` |
| Add contract | `recipes/add-contract.md` + `scripts/create_contract.py` |
| IPC / events | `guides/ipc.md`, `guides/events.md`, `guides/rust-python-ipc.md` |
| Rust ↔ Python | `recipes/add-rust-python-ipc.md` + `scripts/create_rust_python_ipc.py` |
| Role-specific rules | `guides/frontend.md`, `guides/rust.md`, `guides/python.md` |
| Code review | `guides/review.md` |
| Examples | `examples/` |

## AI Coding Rules

- Minimal diff only; one feature at a time
- Analyze impact and role boundary before editing
- Do not modify unrelated files, public APIs, or dependencies
- Do not generate unrequested code or guess requirements

## Full Knowledge Base

All content lives at project root: **`skills/opendesk/`**

# Scripts

OpenDesk 脚手架与架构检查脚本。均需从仓库根目录运行。

## 脚手架

| 脚本 | 用途 |
|------|------|
| `create_feature.py` | 完整 Feature（contract + crate + frontend） |
| `create_crate.py` | Rust crate + workspace 注册 |
| `create_contract.py` | JSON Schema |
| `create_ipc.py` | IPC request/response 对 |
| `create_event.py` | Event schema |
| `create_query_port.py` | ports trait |
| `create_usecase.py` | UseCase 模块 |
| `create_python_package.py` | Python 包 + uv workspace 注册 |
| `create_rust_python_ipc.py` | Rust ↔ Python sidecar IPC 骨架 |
| `sync_contracts.py` | 契约 codegen 占位同步 |

## 检查

| 脚本 | 用途 |
|------|------|
| `check_architecture.py` | **聚合检查（推荐）** |
| `check_layers.py` | Feature UI 禁 @tauri-apps/api |
| `check_boundary.py` | Python 禁 SQLite |
| `check_imports.py` | Feature crate 互依赖 |
| `check_naming.py` | 命名规范 |
| `check_contracts.py` | Schema JSON 合法性 |
| `lint_all.py` | 运行 `pnpm lint` |
| `generate_tree.py` | 项目树 |

## 示例

```bash
python skills/opendesk/scripts/create_feature.py --name myfeature --dry-run
python skills/opendesk/scripts/check_architecture.py
python skills/opendesk/scripts/generate_tree.py --max-depth 3
```

## 扩展

共享工具见 `_common.py`。新增检查脚本后在 `check_architecture.py` 中注册。

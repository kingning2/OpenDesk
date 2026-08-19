# Dependency Rules

## 全局依赖方向

```
React  →  platform  →  (Tauri IPC)  →  Rust  →  ports  ←  infrastructure
                                              ↓
                                           Python
```

禁止任何反向或跨层捷径。

## Rust Workspace 依赖矩阵

业务代码（UseCase / Tauri commands）放在 `apps/desktop/src-tauri`；`crates/` 仅放基础设施，基础设施禁止依赖业务代码。

| Crate 类型 | 可依赖 | 禁止依赖 |
|------------|--------|----------|
| `common` | — | 业务代码（src-tauri） |
| `kernel` | — | 业务代码 |
| `ports` | `common` | 业务代码, `storage`, `runtime`, `adapter` |
| `adapter` | `common`, `ports`, `runtime` | 业务代码 |
| `storage` | `common`, `ports` | 业务代码 |
| `runtime` | `common`, `ports` | 业务代码 |

`src-tauri`（业务层）可依赖：`common` · `kernel` · `ports` · `adapter` · `storage` · `runtime`。

## React 依赖矩阵

| 包 | 可依赖 | 禁止依赖 |
|----|--------|----------|
| `packages/ui` | React, CSS | `@desk/platform`, contracts, features |
| `packages/platform` | `@tauri-apps/api`, contracts | feature 业务逻辑 |
| `features/*` | `ui`, `platform`, `contracts` | `@tauri-apps/api`, 其他 feature 内部 |
| `apps/desktop` | 所有 packages, features | 直接 Tauri（应经 platform） |

## Python 依赖矩阵

Python 包只服务 sidecar 例外能力。不要为默认 AI 增加依赖。

| 包 | 可依赖 | 禁止依赖 |
|----|--------|----------|
| `contracts` | pydantic / typing | sqlalchemy, sqlite3 |
| `shared` | — | tauri, react |
| `gateway` | contracts, shared | tauri, react, SQLite |
| `sidecar` | gateway, contracts, shared | GUI 框架 |

## 循环依赖检测

运行：

```bash
python skills/dingda/scripts/check_imports.py
python skills/dingda/scripts/check_boundary.py
```

## 新增依赖检查清单

- [ ] 是否违反 Layer Boundary？
- [ ] 是否引入 Feature 间耦合？
- [ ] 是否应在 `ports` 而非直接依赖实现 crate？
- [ ] 是否需要在 `Cargo.toml` workspace.dependencies 声明？

## 相关文档

- [layers.md](layers.md)
- [feature-boundary.md](feature-boundary.md)
- [../guides/review.md](../guides/review.md)

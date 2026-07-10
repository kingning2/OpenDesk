# Dependency Rules

## 全局依赖方向

```
React  →  platform  →  (Tauri IPC)  →  Rust  →  ports  ←  infrastructure
                                              ↓
                                           Python
```

禁止任何反向或跨层捷径。

## Rust Workspace 依赖矩阵

| Crate 类型 | 可依赖 | 禁止依赖 |
|------------|--------|----------|
| `kernel` | `common` | feature crates, `storage` 实现 |
| `ports` | `common` | feature crates, `storage`, `runtime` |
| `<feature>` | `common`, `kernel`, `ports` | 其他 `<feature>` |
| `storage` | `common`, `ports` | feature crates |
| `runtime` | `common`, `ports` | feature crates（除 adapter） |
| `app` | 所有注册 feature | — |

## React 依赖矩阵

| 包 | 可依赖 | 禁止依赖 |
|----|--------|----------|
| `packages/ui` | React, CSS | `@desk/platform`, contracts, features |
| `packages/platform` | `@tauri-apps/api`, contracts | feature 业务逻辑 |
| `features/*` | `ui`, `platform`, `contracts` | `@tauri-apps/api`, 其他 feature 内部 |
| `apps/desktop` | 所有 packages, features | 直接 Tauri（应经 platform） |

## Python 依赖矩阵

| 包 | 可依赖 | 禁止依赖 |
|----|--------|----------|
| `contracts` | pydantic / typing | sqlalchemy, sqlite3 |
| `gateway` | contracts, shared | tauri, react |
| `worker` | queue, contracts | 业务持久化 |
| `sidecar` | gateway, 各 AI 包 | GUI 框架 |

## 循环依赖检测

运行：

```bash
python skills/opendesk/scripts/check_imports.py
python skills/opendesk/scripts/check_boundary.py
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

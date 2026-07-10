# Testing Guide

骨架阶段以**可测试性设计**为主，完整测试套件随业务阶段补充。

## 原则

- UseCase 依赖 Port trait → 单元测试注入 Mock
- 集成测试不跨 Layer（不测 React 调 Python）
- Contract 测试：schema 合法性与 codegen 产物一致性

## Rust

```
crates/<feature>/tests/
└── <usecase>_test.rs    # 使用 mockall 或手写 Mock Port
```

```rust
#[cfg(test)]
mod tests {
    struct MockRepo;
    impl ThreadRepo for MockRepo { /* ... */ }
}
```

## TypeScript

- `packages/ui`：组件快照 / 交互测试（Vitest + Testing Library）
- `packages/platform`：IPC 层 mock，不启动 Tauri

## Python

```python
# tests/test_gateway_skeleton.py
def test_health_route_exists():
    assert True  # 骨架占位
```

## 架构测试（必选）

```bash
python skills/opendesk/scripts/check_architecture.py
pnpm lint
```

## 相关

- [review.md](review.md)

# 设计原则

1. **Contract First**：跨端字段先改 `contracts/`，再生成 Rust 与 TypeScript。
2. **Feature First**：按业务 Feature 垂直切分，禁止直接依赖其他 Feature 内部实现。
3. **Dependency Inward**：业务规则依赖 Port，不依赖 IO 实现。
4. **Rust Runtime**：业务、任务、模型和外部协议统一由 Rust 协调。
5. **Event for Writes**：跨域状态传播用 Event；只读查询用 Query Port。
6. **Local First**：业务数据默认保留在本机 SQLite。
7. **Testable by Design**：在 Port 和纯函数边界验证业务，不用框架耦合的测试替代设计。
8. **Explicit over Implicit**：IPC 注册、错误、状态转换和副作用必须显式。

冲突时优先级：

```text
Contract → Feature Boundary → Layer Boundary → Convenience
```

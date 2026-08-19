# Contracts Domain

## 职责

`contracts/` 是跨端 DTO、HTTP、IPC、Event 和 Error 的唯一真相源。实现端默认是 React 与 Rust；Python 仅在 sidecar 例外能力涉及该契约时才更新。

## 变更顺序

```text
Contract → Codegen → 受影响实现端
默认：Rust → React
仅当该能力必须走 sidecar：再改 Python
```

## 管理规则

- Breaking Change 使用新版本并提供迁移说明；
- Change Record 只描述修改意图，字段定义仍以正式 Schema/OpenAPI 为准；
- Contract 变更需要遵守仓库既有共同评审要求；
- 三端不能长期维护手写的重复 DTO。

## 当前状态

已有 Sidecar 管理面和少量 Agent Ping 契约，代码生成能力尚未完整落地。

# Recipe: Add Event

## 修改顺序

1. Contract: `contracts/schema/v1/<f>/event/<name>.schema.json`
2. `sync_contracts.py`
3. Publisher: UseCase 骨架中预留 `publish` 调用点（注释 TODO）
4. Subscriber: 目标 Feature 注册 handler 骨架
5. `check_architecture.py`

## 禁止

- 未定义 payload 就 publish
- Subscriber 中写另一 Feature 的持久化逻辑（Skeleton 阶段）

## Checklist

- [ ] topic 命名 `<feature>.<entity>.<verb>`
- [ ] payload 与 schema 一致
- [ ] 幂等设计已文档化

## 模板

[../templates/event/](../templates/event/)

# Events Guide

## 何时用 Event

- Feature A 的状态变更需要通知 Feature B（无返回值）
- 异步副作用（索引、统计、审计）
- 系统级通知（sidecar 重启）

## 何时不用 Event（改用 Query Port）

- Feature B 需要**同步只读**查询 Feature A 的数据
- 请求-响应语义

## 定义步骤

1. `contracts/schema/v1/<feature>/event/<name>.schema.json`
2. `sync_contracts.py`
3. Publisher：在 UseCase 末尾 `event_bus.publish(topic, payload)`
4. Subscriber：在 feature 启动时 `event_bus.subscribe(topic, handler)`

## Topic 命名

```
<feature>.<entity>.<verb-past-tense>

chat.message.sent
mail.sync.completed
```

## 幂等性

Subscriber 必须 tolerate 重复投递。使用 `event_id` 去重。

## 禁止

- 在 Event handler 中调用另一 Feature 的 UseCase 写操作（应再发 Event 或走 Port）
- 未在 contracts 定义的 payload 字段

## 相关

- [../architecture/event.md](../architecture/event.md)
- [../recipes/add-event.md](../recipes/add-event.md)

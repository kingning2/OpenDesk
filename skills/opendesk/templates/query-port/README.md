# Query Port Template

只读跨 Feature 查询接口。

## 生成

```bash
python skills/opendesk/scripts/create_query_port.py --name chat_query
```

## 规则

- trait 定义在 `crates/ports`
- 实现在提供数据的 feature infra 层
- 仅只读方法

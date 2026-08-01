# Recipe: Add Event

1. 确认是已发生事实或异步状态传播；同步查询改用 Query Port。
2. 在 Contract 定义 payload。
3. 运行 `pnpm contracts:sync`。
4. Publisher 只发布 topic + payload。
5. Subscriber 幂等处理；Tauri Event 单独适配 React。

验证 Contract 同步、重复投递和失败路径。模板见 [`../templates/event/`](../templates/event/)。

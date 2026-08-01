# Event Template

Event payload 先定义在 `contracts/schema/v1/<feature>/event/`，再运行 `pnpm contracts:sync`。

Publisher 不知道 Subscriber；Subscriber 必须幂等。模板文件：[`event.schema.json.tpl`](event.schema.json.tpl)。

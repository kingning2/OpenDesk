# IPC Template

Tauri IPC 契约与封装骨架。

## 结构

```
contracts/schema/v1/<feature>/ipc/<cmd>.request.schema.json
contracts/schema/v1/<feature>/ipc/<cmd>.response.schema.json
packages/platform/src/ipc/<feature>.ts
apps/desktop/src-tauri/src/commands/<feature>.rs
```

## TODO

- [ ] request/response schema 成对
- [ ] platform typed wrapper
- [ ] tauri command 空实现

## 生成

```bash
python skills/opendesk/scripts/create_ipc.py --feature chat --command list_threads
```

# Recipe: Add IPC

## 修改顺序

1. `create_ipc.py --feature X --command Y`
2. `sync_contracts.py`
3. Rust: `#[tauri::command]` 空实现于 `src-tauri`
4. Platform: `packages/platform/src/ipc/<feature>.ts` wrapper 骨架
5. Feature hook 调用 platform wrapper
6. `pnpm lint`

## 禁止

- Feature 直接 invoke
- IPC 返回未在 contract 定义的类型

## Checklist

- [ ] request/response schema 成对
- [ ] 命令名 `<feature>_<action>_<resource>`
- [ ] 错误映射 ErrorDto

## 模板

[../templates/ipc/](../templates/ipc/)

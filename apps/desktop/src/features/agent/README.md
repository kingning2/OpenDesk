# Agent Feature Template

可复制垂直切片：`契约 → @desk/platform/ipc → Feature hook → Feature 页`。

## 文件

| 文件 | 职责 |
|------|------|
| `use-agent-ping.ts` | 调用 platform IPC（禁止 `@tauri-apps/api`） |
| `agent-page.tsx` | Feature UI |
| `index.ts` | Feature 注册导出 |

## 复制为新 Feature

1. 复制目录到 `features/<name>/`
2. 添加 contract + `create_ipc.py`
3. 在 `packages/platform/src/ipc/` 增加 wrapper
4. 注册 Rust Tauri command
5. 在 `route/router.tsx` 增加路由

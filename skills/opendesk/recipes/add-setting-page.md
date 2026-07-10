# Recipe: Add Setting Page

## 修改顺序

1. `features/setting/pages/<name>.tsx`
2. IPC contract: `setting_get_*` / `setting_update_*`（骨架）
3. Platform IPC wrapper
4. Rust 空 command

## 禁止

- 设置项硬编码于 React（应来自 contract DTO）
- 密钥展示于 UI 日志

## 相关

[add-ipc.md](add-ipc.md) · [add-sidebar-page.md](add-sidebar-page.md)

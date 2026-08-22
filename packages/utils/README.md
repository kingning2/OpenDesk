# @desk/utils

零依赖前端工具函数。

## 边界

- **属于**：纯函数工具（日期格式化、错误提取、金额 / 数量格式化）。
- **不属于**：任何依赖 UI / 状态 / IPC 的逻辑。

## Usage

- `formatDateTime` — 中文 `YYYY/MM/DD HH:mm:ss`（空值回退 `-`）
- `getErrorMessage` — 从 Error / string / 响应包装 / Tauri IPC 错误提取可读文案
- `formatAmount` / 计数格式化

## 禁止

- 引入运行时依赖（保持零依赖）。
- 与 React 生命周期 / 组件状态耦合。

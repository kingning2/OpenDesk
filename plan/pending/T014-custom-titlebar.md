| Field | Value |
|-------|-------|
| ID | T014 |
| Priority | P0 |
| Status | pending |
| Depends on | T013 |
| Blocks | T016 |
| Milestone | M5 |

## Goal

实现 Tauri 自定义标题栏（无边框窗口 + 可拖拽区域 + 窗口控制按钮），替换系统原生标题栏。

## Scope

- **Tauri 配置**
  - `tauri.conf.json`：`decorations: false`（保留 `title` 供任务栏识别）
  - `capabilities/default.json`：补充 window 相关权限（如 `core:window:allow-minimize` 等）
- **`@desk/ui`**
  - `TitleBar` 组件：左侧 logo/应用名、中间可拖拽区（`data-tauri-drag-region`）、右侧窗口控制按钮
  - 按平台区分控制按钮布局（Windows 右对齐；macOS 预留 traffic-light 占位区）
- **`@desk/platform`**
  - `src/window/index.ts`：封装 `minimize` / `toggleMaximize` / `close`（基于 `@tauri-apps/api/window`，**不**在 Feature 层直连）
- **`apps/desktop`**
  - `app/shell.tsx` 顶部挂载 `TitleBar`

## Out of scope

- 多窗口 / 子窗口
- 标题栏动态标题同步（由 T015 AppHeader 负责页面级标题）
- Linux 平台特殊适配（占位即可）

## Acceptance criteria

- [ ] 开发模式下窗口无系统原生标题栏，可拖拽移动
- [ ] 最小化 / 最大化（或还原）/ 关闭按钮可用
- [ ] Feature 层无 `@tauri-apps/api` 导入（窗口 API 仅在 platform）
- [ ] `pnpm lint:frontend` 通过

## Key files

- `apps/desktop/src-tauri/tauri.conf.json`
- `apps/desktop/src-tauri/capabilities/default.json`
- `packages/ui/src/components/layout/title-bar.tsx`
- `packages/platform/src/window/index.ts`
- `apps/desktop/src/app/shell.tsx`

## Notes

- 参考 [Tauri v2 Window Customization](https://v2.tauri.app/learn/window-customization/) 的 drag region 与 decorations 配置
- Windows 需在 `globals.css` 或 TitleBar 上处理顶部安全区，避免内容被遮挡

# Frontend Guide（React / TypeScript）

适用范围：`apps/desktop/src/**`、`packages/ui/**`、`packages/platform/**`

## 必须技术栈

| 类别 | 库 |
|------|-----|
| 编译 | **React Compiler**（`babel-plugin-react-compiler`） |
| UI | **shadcn/ui + Radix UI**（组件只在 `packages/ui`） |
| 样式 | **Tailwind CSS**（令牌在 `@desk/ui/tokens`） |
| 动画 | **Motion** + Spring 预设 |
| 图标 | **Lucide React** |
| 表格 | **TanStack Table** |
| 虚拟列表 | **TanStack Virtual** |
| 表单 | **Zod + React Hook Form** |
| 日期 | **date-fns** |
| 主题 | **next-themes** |
| 拖拽 | **dnd-kit** |
| 命令面板 | **cmdk** |
| Toast | **Sonner** |
| Workflow | **Monaco Editor** |

详见 [ui-design-system.md](ui-design-system.md)。

## 样式规则（硬约束）

Feature 层 **禁止** 裸 Tailwind 原子类：

```tsx
// ❌
<div className="bg-white rounded-lg backdrop-blur-xl" />

// ✅
import { Card } from "@desk/ui";
<Card variant="glass" />
```

`packages/ui` 内部用 Tailwind 组装 `variant`；对外只暴露组件 API。

## Apple 风令牌

Glass · Blur · Backdrop · Motion · Spring · Dynamic Color · Radius · Typography

定义于 `packages/ui/src/tokens/`，通过 CSS 变量与 `spring` 预设导出。

## 职责

| 负责 | 禁止 |
|------|------|
| UI · 交互 · 主题 · 布局 · 动画 | 业务规则 · SQL · AI 逻辑 |
| 组合 `@desk/ui` 组件 | Feature 裸 Tailwind / 直接 Radix |
| 通过 platform 调 Rust | `@tauri-apps/api`（Feature 层） |

## 目录约定

```
apps/desktop/src/
├── app/              # 根组件 + globals.css
├── route/
└── features/<name>/

packages/ui/src/
├── tokens/           # 设计令牌
├── components/       # shadcn 风格组件
├── theme/            # ThemeProvider
└── lib/cn.ts
```

## IPC

```typescript
import { ipc } from "@desk/platform/ipc";
// Feature 禁止 invoke() / @tauri-apps/api
```

## shadcn 组件添加

```bash
cd packages/ui && pnpm dlx shadcn@latest add button
```

## Lint

```bash
pnpm lint:frontend
```

## 布局与页面骨架

M5 UI Shell 由 `@desk/ui` 布局原语 + `apps/desktop` 组装层构成：

```
TitleBar（窗口级）
└─ AppLayout
   ├─ Sidebar + SidebarNav
   └─ MainPanel
      ├─ AppHeader（页面级标题 + ThemeToggle）
      └─ PageScaffold / Feature 内容
```

| 组件 | 包 | 用途 |
|------|-----|------|
| `TitleBar` | `@desk/ui` | 无边框窗口拖拽 + 窗口控制（回调由 shell 接 `@desk/platform/window`） |
| `AppLayout` / `Sidebar` / `MainPanel` | `@desk/ui` | 应用级布局 |
| `AppHeader` / `ThemeToggle` | `@desk/ui` | 路由标题 + 主题切换 |
| `PageScaffold` / `PageContainer` | `@desk/ui` | Feature 页面统一内边距与宽度 |
| `nav-registry.ts` | `apps/desktop` | 聚合各 Feature 的 `navItem` |
| `page-meta.ts` | `apps/desktop` | 路由 → 页面标题映射 |

新增 Feature 前端骨架：

1. `features/<name>/index.ts` 导出 `{ id, path, navItem }` 与 page 组件
2. 在 `route/nav-registry.ts` 注册 `navItem`
3. 在 `route/router.tsx` 添加路由
4. 在 `route/page-meta.ts` 添加标题（可选 description）
5. 页面内容用 `PageScaffold` 包裹；占位页可复用 `FeaturePlaceholderPage`

## 相关

- [ui-design-system.md](ui-design-system.md)
- [ipc.md](ipc.md)
- [../../packages/ui/README.md](../../packages/ui/README.md)

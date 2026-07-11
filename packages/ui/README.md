# @desk/ui

OpenDesk 设计系统与 UI 组件库。所有视觉与交互原语**必须**从此包导出。

> Feature 层禁止裸用 `bg-white`、`rounded-lg` 等 Tailwind 原子类；使用语义组件与 variant。

## 技术栈

| 能力 | 库 |
|------|-----|
| 组件 | shadcn/ui + Radix UI |
| 样式 | Tailwind CSS（令牌在 `src/tokens/`） |
| 动画 | Motion（Spring） |
| 图标 | Lucide React |
| 表格 | TanStack Table |
| 虚拟列表 | TanStack Virtual |
| 表单 | Zod + React Hook Form |
| 日期 | date-fns |
| 主题 | next-themes |
| 拖拽 | dnd-kit |
| 命令面板 | cmdk |
| Toast | Sonner |
| Workflow | Monaco Editor |

## Apple 风令牌

| 令牌 | CSS 变量前缀 | 用途 |
|------|--------------|------|
| Glass | `--glass-*` | 半透明材质 |
| Blur | `--blur-*` | 模糊强度 |
| Backdrop | `--backdrop-*` | 背景遮罩 |
| Motion | `--motion-*` | 时长 / 缓动 |
| Spring | `spring.*` in `tokens/motion.ts` | Motion spring 预设 |
| Dynamic Color | `--color-*` | 语义色（非 hex 硬编码） |
| Radius | `--radius-*` | 圆角 |
| Typography | `--font-*` / `--text-*` | 字体阶梯 |

## 使用

桌面端 `globals.css` 必须扫描本包源码，否则布局/组件类不会进入最终 CSS：

```css
@import "tailwindcss";
@import "@desk/ui/tokens";

@source "../../../../packages/ui/src/**/*.{ts,tsx}";
```

```tsx
import { Card, ThemeProvider, Toaster } from "@desk/ui";

export function Page() {
  return (
    <ThemeProvider>
      <Card variant="glass">Content</Card>
      <Toaster />
    </ThemeProvider>
  );
}
```

## 目录

```
src/
├── tokens/          # 设计令牌（CSS 变量 + Motion spring）
├── lib/             # cn() 等工具
├── theme/           # ThemeProvider（next-themes）
└── components/      # shadcn 风格组件（variant 驱动）
```

## 禁止

- IPC / 业务状态 / API 调用
- 在 Feature 中复制本包 Tailwind 类名

## shadcn 初始化

组件通过 shadcn CLI 添加到本包（`components.json`），而非 `apps/desktop`。

```bash
cd packages/ui && pnpm dlx shadcn@latest add button
```

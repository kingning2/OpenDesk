# UI Design System

OpenDesk 采用 **Apple 风** 视觉语言，全部封装在 `@desk/ui`。

## 核心原则

1. **语义组件优先** — Feature 只组合 `@desk/ui`，不写原子 Tailwind
2. **Variant 驱动** — `variant="glass"` 而非复制 class 字符串
3. **令牌单一来源** — 颜色 / 圆角 / 模糊 / 动画来自 `packages/ui/src/tokens/`
4. **React Compiler** — 自动 memo，减少手写优化

## 禁止 vs 必须

```tsx
// ❌ apps/desktop/src/features/** 中禁止
<div className="bg-white dark:bg-zinc-900 rounded-lg border p-4 backdrop-blur-md" />

// ✅ 必须
import { Card } from "@desk/ui";
<Card variant="glass" padding="md">...</Card>
```

`packages/ui` 内部**可以**使用 Tailwind utility 组装 variant，但对外暴露组件 API。

## 技术映射

| 需求 | 封装位置 |
|------|----------|
| 按钮 / 输入 / 对话框 | `@desk/ui` shadcn + Radix |
| 毛玻璃卡片 | `Card variant="glass"` |
| 页面过渡 | `motion` + `spring.default` |
| 深色模式 | `ThemeProvider`（next-themes） |
| 数据表 | `DataTable`（TanStack Table） |
| 长列表 | `VirtualList`（TanStack Virtual） |
| 表单校验 | `Form` + `zodResolver` |
| 日期显示 | `format` from date-fns via ui helpers |
| 拖拽排序 | `Sortable`（dnd-kit） |
| ⌘K 面板 | `Command`（cmdk） |
| 通知 | `toast` / `Toaster`（Sonner） |
| 工作流编辑 | `WorkflowEditor`（Monaco） |

## Glass 示例

```tsx
<Card variant="glass">
  <CardHeader>
    <CardTitle>智能客服</CardTitle>
  </CardHeader>
  <CardContent>...</CardContent>
</Card>
```

对应令牌（`tokens/index.css`）：

```css
--glass-bg: oklch(1 0 0 / 0.55);
--glass-blur: 24px;
--glass-border: oklch(1 0 0 / 0.18);
--glass-shadow: 0 8px 32px oklch(0 0 0 / 0.12);
```

## Motion / Spring

```tsx
import { motion } from "motion/react";
import { spring } from "@desk/ui/tokens/motion";

<motion.div transition={spring.snappy} />
```

## 相关

- [frontend.md](frontend.md)
- [../../packages/ui/README.md](../../packages/ui/README.md)

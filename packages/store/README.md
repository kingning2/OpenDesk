# @desk/store

OpenDesk **通用** 客户端状态底座，基于 [Zustand](https://github.com/pmndrs/zustand)。

## 边界

| 放在 `@desk/store` | 不放在 `@desk/store` |
|--------------------|----------------------|
| `create` / `createDeskStore` / 类型再导出 | Feature 业务状态（`chatStore` 实现放 Feature） |
| 可选通用中间件再导出（按需扩展） | IPC 调用（用 `@desk/platform`） |
| | UI 组件（用 `@desk/ui`） |

## 使用

```ts
import { createDeskStore } from "@desk/store";

interface ExampleState {
  open: boolean;
  setOpen: (open: boolean) => void;
}

/** 示例：Feature 内定义具体 store */
export const exampleStore = createDeskStore<ExampleState>((set) => ({
  open: false,
  setOpen: (open) => set({ open }),
}));

// 组件中
// const open = exampleStore((s) => s.open);
```

也可直接使用 Zustand 原生 API：

```ts
import { create, createStore, useStore } from "@desk/store";
```

## 命名

- Store 实例：`chatStore`、`userStore`（见 `.cursor/rules/frontend.md`）
- Hook：优先用 store 本身作 selector hook，不必再包一层 `useXxx`（除非有组合逻辑）

/**
 * 统一的 Zustand store 工厂。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */

import { create, type StateCreator, type UseBoundStore } from "zustand";
import type { StoreApi } from "zustand";

/**
 * `createDeskStore` 返回的 React store hook 类型。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */
export type DeskStore<T> = UseBoundStore<StoreApi<T>>;

/**
 * 创建 OpenDesk 约定的 Zustand store（React hook）。
 *
 * 命名建议：`chatStore`、`userStore`（见 frontend 规范）。
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @typeParam T - store 状态（含 actions）形状
 * @param initializer - Zustand `StateCreator`
 * @returns 可在组件中调用的 store hook
 *
 * @example
 * ```ts
 * import { createDeskStore } from "@desk/store";
 *
 * interface CounterState {
 *   count: number;
 *   inc: () => void;
 * }
 *
 * export const counterStore = createDeskStore<CounterState>((set) => ({
 *   count: 0,
 *   inc: () => set((s) => ({ count: s.count + 1 })),
 * }));
 * ```
 */
export function createDeskStore<T>(initializer: StateCreator<T>): DeskStore<T> {
  return create<T>()(initializer);
}

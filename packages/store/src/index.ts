/**
 * `@desk/store` — OpenDesk 通用状态底座（Zustand）。
 *
 * 只提供创建 store 的基础设施；业务状态放在各 Feature / 壳层模块。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */

export { create } from "zustand";
export { useStore } from "zustand";
export { createStore } from "zustand/vanilla";
export type { StoreApi, StateCreator, UseBoundStore } from "zustand";

export { createDeskStore } from "./create-desk-store";
export type { DeskStore } from "./create-desk-store";

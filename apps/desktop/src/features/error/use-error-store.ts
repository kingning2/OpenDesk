/**
 * 错误生命周期 store — 后端不可用状态标记。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

import { createDeskStore } from "@desk/store";

export interface ErrorState {
  /** 后端（sidecar）是否不可用。 */
  backendUnavailable: boolean;
  /** 标记后端不可用状态。 */
  setBackendUnavailable: (value: boolean) => void;
}

export const useErrorStore = createDeskStore<ErrorState>((set) => ({
  backendUnavailable: false,
  setBackendUnavailable: (value) => set({ backendUnavailable: value }),
}));

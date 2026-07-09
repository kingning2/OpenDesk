import { invoke } from "@tauri-apps/api/core";

export function call<T>(command: string, payload?: Record<string, unknown>) {
  return invoke<T>(command, payload);
}

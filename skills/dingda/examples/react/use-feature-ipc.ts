/**
 * Example: Feature hook calling platform IPC (best practice).
 * No @tauri-apps/api in features — only @desk/platform/ipc.
 */

// import { ipc } from "@desk/platform/ipc";

export function useExampleThreads() {
  // const list = () => ipc.invoke("chat_list_threads", {});
  return {
    // list,
    status: "skeleton" as const,
  };
}

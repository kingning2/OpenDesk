import { getCurrentWindow } from "@tauri-apps/api/window";

export type DesktopPlatform = "macos" | "windows" | "linux";

export function getPlatform(): DesktopPlatform {
  if (typeof navigator === "undefined") {
    return "windows";
  }

  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("mac")) {
    return "macos";
  }
  if (ua.includes("win")) {
    return "windows";
  }
  return "linux";
}

export async function minimizeWindow(): Promise<void> {
  await getCurrentWindow().minimize();
}

export async function toggleMaximizeWindow(): Promise<void> {
  const window = getCurrentWindow();
  if (await window.isMaximized()) {
    await window.unmaximize();
    return;
  }
  await window.maximize();
}

export async function closeWindow(): Promise<void> {
  await getCurrentWindow().close();
}

export async function startWindowDrag(): Promise<void> {
  await getCurrentWindow().startDragging();
}

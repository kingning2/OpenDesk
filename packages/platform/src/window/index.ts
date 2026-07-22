/**
 * 桌面窗口与平台能力封装（唯一允许使用 `@tauri-apps/api/window` 的入口）。
 *
 * 负责：
 * - 读取 Rust 注入的桌面平台标签
 * - 最小化 / 最大化 / 关闭 / 拖拽
 * - 订阅窗口最大化状态
 *
 * @author coisini
 * @created 2026-07-20
 */

import { getCurrentWindow } from "@tauri-apps/api/window";
import { openUrl } from "@tauri-apps/plugin-opener";

/** 桌面 OS 平台标签（与 Rust `desktop_platform_label` 对齐）。 */
export type DesktopPlatform = "macos" | "windows" | "linux";

/** Rust 注入到 `window` 上的平台全局键。 */
export const OPENDESK_PLATFORM_KEY = "__OPENDESK_PLATFORM__" as const;

declare global {
  interface Window {
    /** 由 Rust `append_invoke_initialization_script` 注入；非 Tauri 环境可能缺失。 */
    __OPENDESK_PLATFORM__?: DesktopPlatform;
  }
}

/**
 * 读取 Rust 注入的桌面平台；无注入时回退为 `windows`（浏览器单独开发预览）。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 桌面平台标签
 */
export function getPlatform(): DesktopPlatform {
  if (typeof window === "undefined") {
    return "windows";
  }

  const injected = window.__OPENDESK_PLATFORM__;
  if (injected === "macos" || injected === "windows" || injected === "linux") {
    return injected;
  }

  return "windows";
}

/**
 * 当前窗口是否已最大化。
 *
 * @author coisini
 * @created 2026-07-21
 *
 * @returns 最大化为 `true`
 */
export async function isWindowMaximized(): Promise<boolean> {
  return getCurrentWindow().isMaximized();
}

/**
 * 订阅窗口最大化状态变化（含初始快照）；返回取消订阅函数。
 *
 * @author coisini
 * @created 2026-07-21
 *
 * @param onChange - 最大化状态回调
 * @returns 取消订阅
 */
export async function subscribeWindowMaximized(
  onChange: (maximized: boolean) => void,
): Promise<() => void> {
  const window = getCurrentWindow();
  onChange(await window.isMaximized());
  return window.onResized(async () => {
    onChange(await window.isMaximized());
  });
}

/**
 * 最小化当前窗口。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 完成后无返回值
 */
export async function minimizeWindow(): Promise<void> {
  await getCurrentWindow().minimize();
}

/**
 * 切换当前窗口最大化 / 还原。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 完成后无返回值
 */
export async function toggleMaximizeWindow(): Promise<void> {
  const window = getCurrentWindow();
  if (await window.isMaximized()) {
    await window.unmaximize();
    return;
  }
  await window.maximize();
}

/**
 * 关闭当前窗口。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 完成后无返回值
 */
export async function closeWindow(): Promise<void> {
  await getCurrentWindow().close();
}

/**
 * 开始拖拽当前窗口。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 完成后无返回值
 */
export async function startWindowDrag(): Promise<void> {
  await getCurrentWindow().startDragging();
}

/**
 * Open an http(s) URL in the system default browser.
 *
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @param url - Absolute URL to open
 * @returns 完成后无返回值
 */
export async function openExternalUrl(url: string): Promise<void> {
  try {
    await openUrl(url);
  } catch {
    if (typeof window !== "undefined") {
      window.open(url, "_blank", "noopener,noreferrer");
    }
  }
}

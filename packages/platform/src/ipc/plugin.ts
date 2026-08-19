/**
 * 插件 IPC 封装 — 列表 / 安装 / 卸载。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */

import type {
  PluginIpcInstallResponse,
  PluginIpcListResponse,
  PluginIpcUninstallResponse,
} from "@desk/contracts";

import { call } from "./invoke";

/**
 * 列出内置插件及安装状态。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @returns 插件列表
 */
export function pluginList(): Promise<PluginIpcListResponse> {
  return call<PluginIpcListResponse>("plugin_list");
}

/**
 * 下载并安装指定插件。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param pluginId - 插件 id（当前仅 `ocr`）
 * @returns 安装后的插件条目
 */
export function pluginInstall(pluginId: string): Promise<PluginIpcInstallResponse> {
  return call<PluginIpcInstallResponse>("plugin_install", {
    request: { plugin_id: pluginId },
  });
}

/**
 * 卸载指定插件的本地文件。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param pluginId - 插件 id
 * @returns 卸载后的插件条目
 */
export function pluginUninstall(pluginId: string): Promise<PluginIpcUninstallResponse> {
  return call<PluginIpcUninstallResponse>("plugin_uninstall", {
    request: { plugin_id: pluginId },
  });
}

/**
 * 设置 Feature 导出。
 *
 * @author coisini
 * @created 2026-07-20
 */

export const settingFeature = {
  id: "setting",
};

export { SettingsDialog } from "./settings-dialog";
export type { SettingsDialogProps } from "./settings-dialog";
export { SettingsDialogProvider } from "./settings-dialog-provider";
export { useSettingsDialog } from "./settings-dialog-store";
export type { SettingsDialogContextValue } from "./settings-dialog-store";
export { useYoutubeApiKeySettings } from "./use-youtube-api-key-settings";

/**
 * 设置弹窗分区注册表 — map 驱动侧栏与内容区渲染。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */

import type { ComponentType } from "react";
import type { LucideIcon } from "@desk/ui/icons";
import { Info, Key, Package } from "@desk/ui/icons";
import { LicenseActivationPanel } from "@feature/license/license-activation-panel";
import { PluginsPanel } from "@feature/plugin";
import { AboutPanel } from "./about-panel";

/** 设置分区 id。 */
export type SettingsSectionId = "license" | "plugins" | "about";

/**
 * 单个设置分区定义。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export interface SettingsSectionDef {
  /** 分区 id。 */
  id: SettingsSectionId;
  /** 侧栏标签。 */
  label: string;
  /** 侧栏图标。 */
  icon: LucideIcon;
  /** 内容面板组件。 */
  Panel: ComponentType;
}

/** 设置弹窗分区列表（顺序即侧栏顺序）。 */
export const SETTINGS_SECTIONS: SettingsSectionDef[] = [
  {
    id: "license",
    label: "激活",
    icon: Key,
    Panel: LicenseActivationPanel,
  },
  {
    id: "plugins",
    label: "插件",
    icon: Package,
    Panel: PluginsPanel,
  },
  {
    id: "about",
    label: "关于",
    icon: Info,
    Panel: AboutPanel,
  },
];

/**
 * 按 id 查找设置分区。
 *
 * @author Xiaoman
 * @created 2026-08-20
 *
 * @param id - 分区 id
 * @returns 分区定义；未找到时返回第一项
 */
export function resolveSettingsSection(id: SettingsSectionId): SettingsSectionDef {
  return SETTINGS_SECTIONS.find((section) => section.id === id) ?? SETTINGS_SECTIONS[0];
}

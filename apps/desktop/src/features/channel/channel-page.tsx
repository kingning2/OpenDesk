/**
 * 渠道平台选择页（一级页）— 编译期仅展示当前构建平台。
 */

import { NavLink } from "react-router";
import { CHANNEL_WORKBENCH_PATH, getActiveChannelPlatform } from "@desk/platform/compile";

import { getChannelPlatform } from "./platforms";

/**
 * 渠道平台选择页。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 渠道页节点
 */
export function ChannelPage() {
  const platform = getChannelPlatform(getActiveChannelPlatform());

  if (!platform) {
    return (
      <>
      <p className="text-(length:--text-sm) text-muted-foreground">多渠道智能客服 — 选择平台</p>
      <p className="text-[length:var(--text-sm)] text-muted-foreground">
          当前构建未配置可用渠道平台。
        </p>
    </>
    );
  }

  const Icon = platform.icon;

  return (
    <>
      <p className="text-(length:--text-sm) text-muted-foreground">多渠道智能客服 — 当前平台</p>
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <NavLink
          to={CHANNEL_WORKBENCH_PATH}
          className="group flex items-start gap-4 rounded-[var(--radius-xl)] border border-border/70 bg-card p-5 transition-colors hover:border-primary/50 hover:bg-muted/30"
        >
          <span className="flex size-11 shrink-0 items-center justify-center rounded-[var(--radius-lg)] bg-primary/10 text-primary">
            <Icon className="size-5" aria-hidden />
          </span>
          <span className="min-w-0">
            <span className="block text-[length:var(--text-base)] font-semibold text-foreground">
              {platform.name}
            </span>
            <span className="mt-1 block text-[length:var(--text-sm)] leading-relaxed text-muted-foreground">
              {platform.description}
            </span>
          </span>
        </NavLink>
      </div>
    </>
  );
}

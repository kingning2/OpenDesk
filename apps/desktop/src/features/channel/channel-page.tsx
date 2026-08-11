/**
 * 渠道平台选择页（一级页）— 列出已支持与规划中的平台。
 */

import { NavLink } from "react-router";
import { PageScaffold } from "@desk/ui";
import { CHANNEL_PLATFORM_COMING_SOON, CHANNEL_PLATFORMS } from "./platforms";

/**
 * 渠道平台选择页。
 */
export function ChannelPage() {
  return (
    <PageScaffold subtitle="多渠道智能客服 — 选择平台">
      <div className="space-y-6">
        <div>
          <h2 className="mb-3 text-[length:var(--text-lg)] font-semibold tracking-tight text-foreground">
            已支持
          </h2>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {CHANNEL_PLATFORMS.map((platform) => {
              const Icon = platform.icon;
              return (
                <NavLink
                  key={platform.kind}
                  to={`/features/channel/${platform.path}`}
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
              );
            })}
          </div>
        </div>

        <div>
          <h2 className="mb-3 text-[length:var(--text-lg)] font-semibold tracking-tight text-foreground">
            规划中
          </h2>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {CHANNEL_PLATFORM_COMING_SOON.map((platform) => {
              const Icon = platform.icon;
              return (
                <div
                  key={platform.kind}
                  className="flex items-start gap-4 rounded-[var(--radius-xl)] border border-dashed border-border/70 bg-card/50 p-5 opacity-60"
                >
                  <span className="flex size-11 shrink-0 items-center justify-center rounded-[var(--radius-lg)] bg-muted text-muted-foreground">
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
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </PageScaffold>
  );
}

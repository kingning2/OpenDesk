import { type ReactNode } from "react";
import { PageGlowCard, cn } from "@desk/ui";
import { type LucideIcon } from "@desk/ui/icons";

/** 客户信息区块 — Aceternity 光晕卡片外壳。 */
export function InfoSection({
  icon: Icon,
  title,
  children,
}: {
  icon: LucideIcon;
  title: string;
  children: ReactNode;
}) {
  return (
    <PageGlowCard className="h-full">
      <div className="relative h-full rounded-[inherit] border border-border bg-card p-4">
        <div className="flex items-center gap-2 text-[length:var(--text-sm)] font-medium">
          <Icon className="size-4 text-primary" aria-hidden />
          <span>{title}</span>
        </div>
        <div className="mt-3 space-y-2">{children}</div>
      </div>
    </PageGlowCard>
  );
}

/** 信息行 — 左标签右值。 */
export function InfoRow({
  label,
  value,
  mono,
}: {
  label: string;
  value?: string | null;
  mono?: boolean;
}) {
  return (
    <div className="flex items-start justify-between gap-3 text-[length:var(--text-xs)]">
      <span className="shrink-0 text-muted-foreground">{label}</span>
      <span className={cn("min-w-0 break-all text-right", mono && "font-mono")}>
        {value || "—"}
      </span>
    </div>
  );
}

/**
 * 授权闸门锁图标居中展示（圆形按钮 + 文案）。
 *
 * 用于启动校验、未激活遮罩等场景，统一锁动画视觉。
 *
 * @author coisini
 * @created 2026-07-16
 */

import { cn } from "@desk/ui";
import { LicenseLockGlyph, type LicenseLockAnim } from "./license-lock-glyph";

/**
 * 锁居中展示组件属性。
 *
 * @author coisini
 * @created 2026-07-16
 */
export interface LicenseLockHeroProps {
  /** 锁动画相位。 */
  anim: LicenseLockAnim;
  /** 锁下方说明文案。 */
  caption: string;
  /** 点击锁按钮；未提供时为不可点击展示。 */
  onLockClick?: () => void;
  /** 是否禁用点击。 */
  disabled?: boolean;
  /** 是否展开激活面板（影响 aria）。 */
  expanded?: boolean;
}

/**
 * 圆形锁按钮与说明文案。
 *
 * @author coisini
 * @created 2026-07-16
 *
 * @param props - 组件属性
 * @returns 锁展示 React 节点
 */
export function LicenseLockHero({
  anim,
  caption,
  onLockClick,
  disabled = false,
  expanded = false,
}: LicenseLockHeroProps) {
  const isBusy = anim === "busy";
  const isSuccess = anim === "success";
  const isFailure = anim === "failure";
  const interactive = Boolean(onLockClick) && !disabled;

  const lockShell = (
    <>
      {isBusy ? (
        <span
          className="pointer-events-none absolute inset-0 animate-ping rounded-full border border-primary/30"
          aria-hidden
        />
      ) : null}
      <LicenseLockGlyph anim={anim} className="size-14" />
    </>
  );

  return (
    <div className="flex flex-col items-center gap-4">
      {interactive ? (
        <button
          type="button"
          onClick={onLockClick}
          disabled={disabled}
          className={cn(
            "relative flex size-28 items-center justify-center rounded-full border border-border/60 bg-card/80 shadow-lg transition hover:bg-card",
            isFailure && "border-foreground/50",
            isSuccess && "border-transparent bg-card/40",
            disabled && "cursor-default",
          )}
          aria-expanded={expanded}
          aria-label={expanded ? "收起激活面板" : "打开激活面板"}
        >
          {lockShell}
        </button>
      ) : (
        <div
          className={cn(
            "relative flex size-28 items-center justify-center rounded-full border border-border/60 bg-card/80 shadow-lg",
            isBusy && "border-primary/40",
            isFailure && "border-foreground/50",
            isSuccess && "border-transparent bg-card/40",
          )}
          role="img"
          aria-label={caption}
        >
          {lockShell}
        </div>
      )}

      <p
        className={cn(
          "text-center text-[length:var(--text-sm)]",
          isFailure ? "text-destructive" : "text-muted-foreground",
          isSuccess && "text-foreground",
        )}
        role="status"
        aria-live="polite"
      >
        {caption}
      </p>
    </div>
  );
}

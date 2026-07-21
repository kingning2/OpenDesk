/**
 * 侧栏底部套餐入口（按钮 + 详情弹窗）。
 *
 * @author coisini
 * @created 2026-07-16
 */

import { useEffect, useState } from "react";
import { cn } from "@desk/ui";
import { Lock } from "@desk/ui/icons";
import {
  formatLicenseRemaining,
  formatLicenseRemainingShort,
} from "./format-license-remaining";
import { useLicenseGateContext } from "./license-gate-context";
import { LicensePlanDialog } from "./license-plan-dialog";

/** 剩余时长刷新间隔（毫秒）。 */
const TICK_MS = 60_000;

/**
 * 侧栏左下角套餐按钮；点击打开完整信息弹窗。
 *
 * 仅在「有锁 + 已激活 + 有 expiresAt」时渲染。
 *
 * @author coisini
 * @created 2026-07-16
 *
 * @returns 入口节点；不满足条件时返回 `null`
 */
export function LicensePlanBadge() {
  const { status, loading, gateBlocks } = useLicenseGateContext();
  const [open, setOpen] = useState(false);
  const [nowSec, setNowSec] = useState(() => Math.floor(Date.now() / 1000));

  useEffect(() => {
    const timer = window.setInterval(() => {
      setNowSec(Math.floor(Date.now() / 1000));
    }, TICK_MS);
    return () => window.clearInterval(timer);
  }, []);

  if (loading || gateBlocks || !status?.gateEnabled || !status.activated) {
    return null;
  }

  const expiresAt = status.expiresAt;
  if (expiresAt == null) {
    return null;
  }

  const remaining = formatLicenseRemaining(expiresAt, nowSec);
  if (!remaining) {
    return null;
  }

  const shortLabel = formatLicenseRemainingShort(expiresAt, nowSec);

  return (
    <>
      <div className="mt-auto w-full px-1.5 pb-2">
        <button
          type="button"
          onClick={() => setOpen(true)}
          className={cn(
            "flex w-full cursor-pointer flex-col items-center gap-0.5 rounded-[var(--radius-md)] px-1 py-2 text-[10px] leading-none transition-colors",
            remaining.expired
              ? "text-destructive hover:bg-destructive/10"
              : remaining.urgent
                ? "text-amber-200 hover:bg-amber-500/10"
                : "text-muted-foreground hover:bg-muted/40 hover:text-foreground",
          )}
          aria-haspopup="dialog"
          aria-expanded={open}
          title={`套餐 ${remaining.text}`}
        >
          <Lock className="size-[1.125rem] shrink-0" strokeWidth={1.5} aria-hidden />
          <span className="max-w-full truncate">套餐</span>
          <span className="max-w-full truncate font-medium">{shortLabel}</span>
        </button>
      </div>

      <LicensePlanDialog
        open={open}
        onClose={() => setOpen(false)}
        status={status}
        remaining={remaining}
        nowSec={nowSec}
      />
    </>
  );
}

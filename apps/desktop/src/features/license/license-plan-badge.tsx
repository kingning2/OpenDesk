/**
 * 侧栏底部套餐 / 激活入口（按钮 + 详情弹窗）。
 *
 * @author coisini
 * @created 2026-07-16
 */

import { useEffect, useState } from "react";
import { Button, cn } from "@desk/ui";
import { Lock } from "@desk/ui/icons";
import { useSettingsDialog } from "@feature/setting";
import {
  formatLicenseRemaining,
  formatLicenseRemainingShort,
} from "./format-license-remaining";
import { useLicenseGateContext } from "./license-gate-context";
import { LicensePlanDialog } from "./license-plan-dialog";

/** 剩余时长刷新间隔（毫秒）。 */
const TICK_MS = 60_000;

/**
 * 侧栏左下角授权入口。
 *
 * - 未激活：打开设置 → 激活
 * - 已激活且有到期时间：展示剩余时长
 *
 * @author coisini
 * @created 2026-07-16
 *
 * @returns 入口节点；无授权闸门时不渲染
 */
export function LicensePlanBadge() {
  const { status, loading, gateBlocks } = useLicenseGateContext();
  const { openSettings } = useSettingsDialog();
  const [open, setOpen] = useState(false);
  const [nowSec, setNowSec] = useState(() => Math.floor(Date.now() / 1000));

  useEffect(() => {
    const timer = window.setInterval(() => {
      setNowSec(Math.floor(Date.now() / 1000));
    }, TICK_MS);
    return () => window.clearInterval(timer);
  }, []);

  if (loading || !status?.gateEnabled) {
    return null;
  }

  if (gateBlocks) {
    return (
      <div className="w-full px-1.5 pb-2">
        <Button
          variant="ghost"
          onClick={() => openSettings("license")}
          className="h-auto w-full flex-col items-center gap-0.5 px-1 py-2 text-[10px] leading-none text-amber-200 hover:bg-amber-500/10"
          title="激活付费功能"
        >
          <Lock className="size-[1.125rem] shrink-0" strokeWidth={1.5} aria-hidden />
          <span className="max-w-full truncate">未激活</span>
          <span className="max-w-full truncate font-medium">去设置</span>
        </Button>
      </div>
    );
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
      <div className="w-full px-1.5 pb-2">
        <Button
          variant="ghost"
          onClick={() => setOpen(true)}
          className={cn(
            "h-auto w-full flex-col items-center gap-0.5 px-1 py-2 text-[10px] leading-none",
            remaining.expired
              ? "text-destructive hover:bg-destructive/10"
              : remaining.urgent
                ? "text-amber-200 hover:bg-amber-500/10"
                : "text-muted-foreground",
          )}
          aria-haspopup="dialog"
          aria-expanded={open}
          title={`套餐 ${remaining.text}`}
        >
          <Lock className="size-[1.125rem] shrink-0" strokeWidth={1.5} aria-hidden />
          <span className="max-w-full truncate">套餐</span>
          <span className="max-w-full truncate font-medium">{shortLabel}</span>
        </Button>
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

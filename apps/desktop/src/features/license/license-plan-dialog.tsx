/**
 * 套餐详情弹窗。
 *
 * @author coisini
 * @created 2026-07-16
 */

import { useEffect } from "react";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  cn,
} from "@desk/ui";
import { X } from "@desk/ui/icons";
import type { LicenseStatus } from "@desk/platform/ipc/license";
import {
  formatLicenseExpiresAt,
  formatLicenseRemainingDetailed,
  type LicenseRemainingLabel,
} from "./format-license-remaining";

/**
 * 套餐详情弹窗属性。
 *
 * @author coisini
 * @created 2026-07-16
 */
export interface LicensePlanDialogProps {
  /** 是否打开。 */
  open: boolean;
  /** 关闭回调。 */
  onClose: () => void;
  /** 授权状态。 */
  status: LicenseStatus;
  /** 剩余时长摘要。 */
  remaining: LicenseRemainingLabel;
  /** 当前 Unix 秒，用于计算详细剩余时长。 */
  nowSec: number;
}

/**
 * 详情行。
 *
 * @author coisini
 * @created 2026-07-16
 */
function DetailRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="space-y-1">
      <dt className="text-[length:var(--text-sm)] text-muted-foreground">{label}</dt>
      <dd className="break-all font-mono text-[length:var(--text-sm)] text-foreground">{value}</dd>
    </div>
  );
}

/**
 * 展示完整套餐与授权信息的模态弹窗。
 *
 * @author coisini
 * @created 2026-07-16
 *
 * @param props - 弹窗属性
 * @returns 弹窗节点；`open=false` 时返回 `null`
 */
export function LicensePlanDialog({
  open,
  onClose,
  status,
  remaining,
  nowSec,
}: LicensePlanDialogProps) {
  useEffect(() => {
    if (!open) return;
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") onClose();
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [open, onClose]);

  if (!open || status.expiresAt == null) {
    return null;
  }

  const product = status.product?.trim() || "OpenDesk";
  const expiresAt = status.expiresAt;
  const detailedRemaining = formatLicenseRemainingDetailed(expiresAt, nowSec);
  const expiresLabel = formatLicenseExpiresAt(expiresAt);
  const machineCode = status.machineCode?.trim() || "—";
  const authStatus = remaining.expired ? "已过期" : status.activated ? "已激活" : "未激活";

  async function copyMachineCode() {
    if (!status.machineCode?.trim()) return;
    try {
      await navigator.clipboard.writeText(status.machineCode.trim());
    } catch {
      // 复制失败时静默；用户仍可手动选择
    }
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/45 p-4"
      role="presentation"
      onClick={onClose}
    >
      <Card
        variant="dialog"
        padding="md"
        className="relative w-full max-w-md"
        role="dialog"
        aria-modal="true"
        aria-labelledby="license-plan-dialog-title"
        onClick={(event) => event.stopPropagation()}
      >
        <button
          type="button"
          onClick={onClose}
          className="absolute right-3 top-3 rounded-[var(--radius-md)] p-1 text-muted-foreground transition hover:bg-muted/40 hover:text-foreground"
          aria-label="关闭"
        >
          <X className="size-4" aria-hidden />
        </button>

        <CardHeader className="pr-10">
          <CardTitle id="license-plan-dialog-title">套餐与授权</CardTitle>
        </CardHeader>

        <CardContent className="space-y-4">
          <dl className="space-y-3">
            <DetailRow label="产品" value={product} />
            <DetailRow label="授权状态" value={authStatus} />
            <DetailRow label="剩余时长" value={detailedRemaining} />
            <DetailRow label="到期时间" value={expiresLabel} />
            <div className="space-y-1">
              <dt className="text-[length:var(--text-sm)] text-muted-foreground">设备码</dt>
              <dd className="flex items-start gap-2">
                <span className="min-w-0 flex-1 break-all font-mono text-[length:var(--text-sm)] text-foreground">
                  {machineCode}
                </span>
                {status.machineCode?.trim() ? (
                  <button
                    type="button"
                    onClick={() => void copyMachineCode()}
                    className="shrink-0 rounded-[var(--radius-md)] bg-secondary px-2 py-1 text-[length:var(--text-sm)]"
                  >
                    复制
                  </button>
                ) : null}
              </dd>
            </div>
          </dl>

          <p
            className={cn(
              "rounded-[var(--radius-md)] border px-3 py-2 text-center text-[length:var(--text-sm)]",
              remaining.expired
                ? "border-destructive/40 bg-destructive/10 text-destructive"
                : remaining.urgent
                  ? "border-amber-500/40 bg-amber-500/10 text-amber-100"
                  : "border-border/60 bg-muted/20 text-muted-foreground",
            )}
          >
            {remaining.text}
          </p>
        </CardContent>
      </Card>
    </div>
  );
}

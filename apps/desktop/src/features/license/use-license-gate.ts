/**
 * License 闸门 React Hook（薄适配层）。
 *
 * 将 [`LicenseGateController`] 接到组件状态。
 *
 * @author Xiaoman
 * @created 2026-07-16
 */

import { useEffect, useState } from "react";
import { LicenseGateController } from "./license-gate-controller";
import type { LicenseStatus } from "@desk/platform/ipc/license";

/**
 * Hook 返回值：闸门加载态与刷新入口。
 *
 * @author Xiaoman
 * @created 2026-07-16
 */
export interface UseLicenseGateResult {
  /** 最新授权状态。 */
  status: LicenseStatus | null;
  /** 是否正在拉取。 */
  loading: boolean;
  /** 拉取失败时的错误消息。 */
  error: string | null;
  /** 是否应展示激活遮罩。 */
  gateBlocks: boolean;
  /** 重新拉取状态。 */
  refresh: () => void;
}

/**
 * 订阅授权闸门状态。
 *
 * @author Xiaoman
 * @created 2026-07-16
 *
 * @returns 闸门 UI 所需状态与 refresh
 */
export function useLicenseGate(): UseLicenseGateResult {
  const [controller] = useState(() => new LicenseGateController());
  const [status, setStatus] = useState<LicenseStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [gateBlocks, setGateBlocks] = useState(false);
  const [reloadToken, setReloadToken] = useState(0);

  useEffect(() => {
    let cancelled = false;
    void controller.fetchSnapshot().then((snapshot) => {
      if (cancelled) return;
      setStatus(snapshot.status);
      setError(snapshot.error);
      setGateBlocks(snapshot.gateBlocks);
      setLoading(false);
    });
    return () => {
      cancelled = true;
    };
  }, [controller, reloadToken]);

  function refresh() {
    setLoading(true);
    setReloadToken((value) => value + 1);
  }

  return { status, loading, error, gateBlocks, refresh };
}

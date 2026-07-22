/**
 * License 激活面板 React Hook（薄适配层）。
 *
 * 将 [`LicenseActivationService`] 接到表单状态，并用 toast + 锁动画反馈每次操作。
 *
 * @author coisini
 * @created 2026-07-16
 */

import { useEffect, useState } from "react";
import { toast } from "@desk/ui";
import { LicenseActivationService } from "./license-activation-service";
import type { LicenseLockAnim } from "./license-lock-glyph";

/** 成功裂开动画时长（毫秒），结束后再卸载闸门。 */
const SUCCESS_ANIM_MS = 900;
/** 失败加固动画时长（毫秒），结束后回到 idle。 */
const FAILURE_ANIM_MS = 700;

/**
 * Hook 返回值：激活表单状态与操作。
 *
 * @author coisini
 * @created 2026-07-16
 */
export interface UseLicenseActivateResult {
  /** 设备码展示值。 */
  machineCode: string;
  /** 当前 token 输入。 */
  token: string;
  /** 更新 token。 */
  setToken: (value: string) => void;
  /** 是否正在激活。 */
  busy: boolean;
  /** 锁图标动画相位。 */
  lockAnim: LicenseLockAnim;
  /** 提示/错误消息。 */
  message: string | null;
  /** 复制设备码。 */
  copyMachineCode: () => Promise<void>;
  /** 用 token 激活。 */
  activateWithToken: () => Promise<void>;
  /** 用 key 文件激活。 */
  activateWithKeyFile: (file: File) => Promise<void>;
}

/**
 * 等待指定毫秒；尊重 `prefers-reduced-motion` 时几乎立即返回。
 *
 * @author coisini
 * @created 2026-07-16
 *
 * @param ms - 正常动画等待时长
 * @returns 无
 */
function waitForAnim(ms: number): Promise<void> {
  const reduced =
    typeof window !== "undefined" &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  return new Promise((resolve) => {
    window.setTimeout(resolve, reduced ? 80 : ms);
  });
}

/**
 * 管理激活面板交互。
 *
 * @author coisini
 * @created 2026-07-16
 *
 * @param onActivated - 激活成功后的回调（通常用于刷新闸门）
 * @returns 面板绑定所需状态与方法
 */
export function useLicenseActivate(
  onActivated: () => void,
): UseLicenseActivateResult {
  const [service] = useState(() => new LicenseActivationService());
  const [machineCode, setMachineCode] = useState("");
  const [token, setToken] = useState("");
  const [lockAnim, setLockAnim] = useState<LicenseLockAnim>("idle");
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void service
      .loadMachineCode()
      .then((code) => {
        if (!cancelled) setMachineCode(code);
      })
      .catch((error: unknown) => {
        if (cancelled) return;
        const text = error instanceof Error ? error.message : String(error);
        setMessage(text);
        toast.error(text);
      });
    return () => {
      cancelled = true;
    };
  }, [service]);

  async function copyMachineCode() {
    const nextMessage = await service.copyMachineCode();
    setMessage(nextMessage);
    if (nextMessage === "设备码已复制") {
      toast.success(nextMessage);
    } else {
      toast.error(nextMessage);
    }
  }

  async function runActivate(
    run: () => ReturnType<LicenseActivationService["activateWithToken"]>,
  ) {
    const pending = "正在校验激活码…";
    setLockAnim("busy");
    setMessage(pending);
    const toastId = toast.loading(pending);
    const result = await run();
    setMessage(result.message);

    if (result.ok) {
      setLockAnim("success");
      toast.success(result.message, { id: toastId });
      await waitForAnim(SUCCESS_ANIM_MS);
      onActivated();
      return;
    }

    setLockAnim("failure");
    toast.error(result.message, { id: toastId });
    await waitForAnim(FAILURE_ANIM_MS);
    setLockAnim("idle");
  }

  async function activateWithToken() {
    await runActivate(() => service.activateWithToken(token));
  }

  async function activateWithKeyFile(file: File) {
    await runActivate(() => service.activateWithKeyFile(file));
  }

  const busy = lockAnim === "busy";

  return {
    machineCode,
    token,
    setToken,
    busy,
    lockAnim,
    message,
    copyMachineCode,
    activateWithToken,
    activateWithKeyFile,
  };
}

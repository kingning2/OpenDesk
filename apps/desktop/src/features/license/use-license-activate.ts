/**
 * License 激活面板 React Hook（薄适配层）。
 *
 * 将 [`LicenseActivationService`] 接到表单状态。
 *
 * @author Xiaoman
 * @created 2026-07-16
 */

import { useEffect, useState } from "react";
import { LicenseActivationService } from "./license-activation-service";

/**
 * Hook 返回值：激活表单状态与操作。
 *
 * @author Xiaoman
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
 * 管理激活面板交互。
 *
 * @author Xiaoman
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
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void service
      .loadMachineCode()
      .then((code) => {
        if (!cancelled) setMachineCode(code);
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setMessage(error instanceof Error ? error.message : String(error));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [service]);

  async function copyMachineCode() {
    const nextMessage = await service.copyMachineCode();
    setMessage(nextMessage);
  }

  async function activateWithToken() {
    setBusy(true);
    setMessage(null);
    const result = await service.activateWithToken(token);
    setMessage(result.message);
    setBusy(false);
    if (result.ok) {
      onActivated();
    }
  }

  async function activateWithKeyFile(file: File) {
    setBusy(true);
    setMessage(null);
    const result = await service.activateWithKeyFile(file);
    setMessage(result.message);
    setBusy(false);
    if (result.ok) {
      onActivated();
    }
  }

  return {
    machineCode,
    token,
    setToken,
    busy,
    message,
    copyMachineCode,
    activateWithToken,
    activateWithKeyFile,
  };
}

/**
 * 未激活时的模糊遮罩与激活面板。
 *
 * @author Xiaoman
 * @created 2026-07-16
 */

import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@desk/ui";
import { Lock } from "@desk/ui/icons";
import { useLicenseActivate } from "./use-license-activate";

/**
 * 激活遮罩组件属性。
 *
 * @author Xiaoman
 * @created 2026-07-16
 */
export interface LicenseLockOverlayProps {
  /** 激活成功后的回调。 */
  onActivated: () => void;
}

/**
 * 全屏模糊遮罩 + 可展开的激活卡片。
 *
 * @author Xiaoman
 * @created 2026-07-16
 *
 * @param props - 组件属性
 * @returns 激活遮罩 React 节点
 */
export function LicenseLockOverlay({ onActivated }: LicenseLockOverlayProps) {
  const [expanded, setExpanded] = useState(false);
  const {
    machineCode,
    token,
    setToken,
    busy,
    message,
    copyMachineCode,
    activateWithToken,
    activateWithKeyFile,
  } = useLicenseActivate(onActivated);

  return (
    <div
      className="fixed inset-x-0 bottom-0 top-11 z-50 flex items-center justify-center bg-background/40 p-6 backdrop-blur-md"
      role="dialog"
      aria-modal="true"
      aria-label="软件激活"
    >
      <div className="flex w-full max-w-md flex-col items-center gap-4">
        <button
          type="button"
          onClick={() => setExpanded((value) => !value)}
          className="flex size-24 items-center justify-center rounded-full border border-border/60 bg-card/80 shadow-lg transition hover:bg-card"
          aria-expanded={expanded}
          aria-label={expanded ? "收起激活面板" : "打开激活面板"}
        >
          <Lock className="size-10 text-muted-foreground" strokeWidth={1.5} aria-hidden />
        </button>

        {expanded ? (
          <Card variant="glass" className="w-full">
            <CardHeader className="text-center">
              <CardTitle>激活 OpenDesk</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="text-center text-[length:var(--text-sm)] text-muted-foreground">
                将设备码发给管理员，导入 `.key` 或粘贴 token 完成激活。
              </p>

              <div className="space-y-2">
                <label className="text-[length:var(--text-sm)] font-medium">设备码</label>
                <div className="flex gap-2">
                  <input
                    readOnly
                    value={machineCode || "加载中…"}
                    className="min-w-0 flex-1 rounded-[var(--radius-md)] border border-border bg-background px-3 py-2 font-mono text-[length:var(--text-sm)]"
                  />
                  <button
                    type="button"
                    onClick={() => void copyMachineCode()}
                    disabled={!machineCode}
                    className="rounded-[var(--radius-md)] bg-secondary px-3 py-2 text-[length:var(--text-sm)] disabled:opacity-60"
                  >
                    复制
                  </button>
                </div>
              </div>

              <div className="space-y-2">
                <label className="text-[length:var(--text-sm)] font-medium">激活 Token</label>
                <textarea
                  value={token}
                  onChange={(event) => setToken(event.target.value)}
                  rows={3}
                  placeholder="粘贴 activation token"
                  className="w-full rounded-[var(--radius-md)] border border-border bg-background px-3 py-2 font-mono text-[length:var(--text-sm)]"
                />
                <button
                  type="button"
                  disabled={busy || !token.trim()}
                  onClick={() => void activateWithToken()}
                  className="w-full rounded-[var(--radius-md)] bg-primary px-4 py-2 text-[length:var(--text-sm)] text-primary-foreground disabled:opacity-60"
                >
                  {busy ? "激活中…" : "用 Token 激活"}
                </button>
              </div>

              <div className="space-y-2">
                <label className="text-[length:var(--text-sm)] font-medium">或导入 .key 文件</label>
                <input
                  type="file"
                  accept=".key,application/octet-stream"
                  disabled={busy}
                  onChange={(event) => {
                    const file = event.target.files?.[0];
                    if (file) void activateWithKeyFile(file);
                  }}
                  className="block w-full text-[length:var(--text-sm)]"
                />
              </div>

              {message ? (
                <p className="text-center text-[length:var(--text-sm)] text-muted-foreground">
                  {message}
                </p>
              ) : null}
            </CardContent>
          </Card>
        ) : (
          <p className="text-[length:var(--text-sm)] text-muted-foreground">
            点击锁图标激活软件
          </p>
        )}
      </div>
    </div>
  );
}

/**
 * 未激活时的模糊遮罩与激活面板。
 *
 * @author coisini
 * @created 2026-07-16
 */

import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle, cn } from "@desk/ui";
import { LicenseLockHero } from "./license-lock-hero";
import { useLicenseActivate } from "./use-license-activate";

/**
 * 激活遮罩组件属性。
 *
 * @author coisini
 * @created 2026-07-16
 */
export interface LicenseLockOverlayProps {
  /** 激活成功后的回调。 */
  onActivated: () => void;
}

/**
 * 根据锁动画相位生成锁下方文案。
 *
 * @author coisini
 * @created 2026-07-16
 *
 * @param lockAnim - 锁动画相位
 * @param expanded - 是否已展开激活面板
 * @returns 说明文案
 */
function lockCaption(
  lockAnim: "idle" | "busy" | "success" | "failure",
  expanded: boolean,
): string {
  if (lockAnim === "success") return "解锁成功";
  if (lockAnim === "failure") return "校验未通过，锁已加固";
  if (lockAnim === "busy") return "正在校验激活码…";
  if (expanded) return "填写激活信息";
  return "点击锁图标激活软件";
}

/**
 * 全屏模糊遮罩 + 可展开的激活卡片。
 *
 * @author coisini
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
    lockAnim,
    message,
    copyMachineCode,
    activateWithToken,
    activateWithKeyFile,
  } = useLicenseActivate(onActivated);

  const animatingResult = lockAnim === "success" || lockAnim === "failure";

  return (
    <div
      className="fixed inset-x-0 bottom-0 top-11 z-50 flex items-center justify-center bg-background/40 p-6 backdrop-blur-md"
      role="dialog"
      aria-modal="true"
      aria-label="软件激活"
    >
      <div className="flex w-full max-w-md flex-col items-center gap-4">
        <LicenseLockHero
          anim={lockAnim}
          caption={lockCaption(lockAnim, expanded)}
          expanded={expanded}
          disabled={busy || animatingResult}
          onLockClick={() => {
            if (busy || animatingResult) return;
            setExpanded((value) => !value);
          }}
        />

        {expanded && lockAnim !== "success" ? (
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
                    disabled={!machineCode || busy || animatingResult}
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
                  disabled={busy || animatingResult}
                  className="w-full rounded-[var(--radius-md)] border border-border bg-background px-3 py-2 font-mono text-[length:var(--text-sm)] disabled:opacity-60"
                />
                <button
                  type="button"
                  disabled={busy || animatingResult || !token.trim()}
                  onClick={() => void activateWithToken()}
                  className="w-full rounded-[var(--radius-md)] bg-primary px-4 py-2 text-[length:var(--text-sm)] text-primary-foreground disabled:opacity-60"
                >
                  {busy ? "校验中…" : "用 Token 激活"}
                </button>
              </div>

              <div className="space-y-2">
                <label className="text-[length:var(--text-sm)] font-medium">或导入 .key 文件</label>
                <input
                  type="file"
                  accept=".key,application/octet-stream"
                  disabled={busy || animatingResult}
                  onChange={(event) => {
                    const file = event.target.files?.[0];
                    if (file) void activateWithKeyFile(file);
                    event.target.value = "";
                  }}
                  className="block w-full text-[length:var(--text-sm)]"
                />
              </div>

              {message ? (
                <p
                  role="status"
                  aria-live="polite"
                  className={cn(
                    "text-center text-[length:var(--text-sm)]",
                    busy
                      ? "text-muted-foreground"
                      : message.includes("成功") || message.includes("已复制")
                        ? "text-foreground"
                        : "text-destructive",
                  )}
                >
                  {message}
                </p>
              ) : null}
            </CardContent>
          </Card>
        ) : null}
      </div>
    </div>
  );
}

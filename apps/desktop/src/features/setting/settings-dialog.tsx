/**
 * 设置弹窗 — AI 账号配置（单栏）。
 *
 * @author coisini
 * @created 2026-07-21
 */

import { useState } from "react";
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
  IconButton,
} from "@desk/ui";
import { X } from "@desk/ui/icons";
import { AiSettingsPanel } from "@feature/ai";

/**
 * `SettingsDialog` 属性。
 *
 * @author coisini
 * @created 2026-07-21
 */
export interface SettingsDialogProps {
  /** 是否打开。 */
  open: boolean;
  /** 打开状态变更。 */
  onOpenChange: (open: boolean) => void;
}

/**
 * 应用设置弹窗。
 *
 * AI 设置即时写入本机；关闭前弹出确认作为临别提示。
 *
 * @author coisini
 * @created 2026-07-21
 *
 * @param props - 见 {@link SettingsDialogProps}
 * @returns 设置 Dialog 节点
 */
export function SettingsDialog({ open, onOpenChange }: SettingsDialogProps) {
  const [confirmExit, setConfirmExit] = useState(false);

  /**
   * 真正关闭弹窗，并清理确认态。
   *
   * @author coisini
   * @created 2026-07-21
   */
  function finishExit() {
    setConfirmExit(false);
    onOpenChange(false);
  }

  /**
   * 请求关闭；总是先弹确认提示。
   *
   * @author coisini
   * @created 2026-07-21
   */
  function requestExit() {
    setConfirmExit(true);
  }

  /**
   * Radix open 变更：拦截关闭以做确认。
   *
   * @author coisini
   * @created 2026-07-21
   *
   * @param next - 下一 open 状态
   */
  function handleOpenChange(next: boolean) {
    if (next) {
      onOpenChange(true);
      return;
    }
    requestExit();
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent
        className="h-[min(780px,92vh)] w-[min(1100px,96vw)] max-w-none gap-0 p-0"
        closeLabel="关闭"
        dismissOnOutsidePress={false}
        showClose={false}
        onEscapeKeyDown={(event) => {
          event.preventDefault();
          requestExit();
        }}
      >
        <DialogTitle className="sr-only">设置</DialogTitle>
        <DialogDescription className="sr-only">应用设置</DialogDescription>

        <IconButton
          label="关闭"
          className="absolute right-3 top-3 z-20"
          onClick={() => requestExit()}
        >
          <X className="size-3.5" aria-hidden />
        </IconButton>

        <div className="relative flex h-full min-h-0">
          <div className="flex min-w-0 flex-1 flex-col">
            <header className="flex shrink-0 items-center border-b border-border/70 px-8 py-5 pr-14">
              <h2 className="font-display text-[length:var(--text-xl)] font-semibold tracking-tight text-foreground">
                AI 账号
              </h2>
            </header>

            <div className="min-h-0 flex-1 overflow-y-auto px-8 py-7">
              <AiSettingsPanel />
            </div>
          </div>

          {confirmExit ? (
            <div
              className="absolute inset-0 z-30 flex items-center justify-center bg-black/50 p-6"
              role="presentation"
            >
              <div
                role="alertdialog"
                aria-modal="true"
                aria-labelledby="settings-confirm-title"
                aria-describedby="settings-confirm-desc"
                className="w-full max-w-sm rounded-[var(--radius-xl)] border border-border bg-card p-6 shadow-[var(--glass-shadow)]"
              >
                <h3
                  id="settings-confirm-title"
                  className="font-display text-[length:var(--text-lg)] font-semibold tracking-tight"
                >
                  设置已即时保存
                </h3>
                <p
                  id="settings-confirm-desc"
                  className="mt-2 text-[length:var(--text-sm)] leading-relaxed text-muted-foreground"
                >
                  设置会立即写入本机，确定关闭吗?
                </p>
                <div className="mt-6 flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
                  <Button
                    type="button"
                    variant="ghost"
                    onClick={() => {
                      setConfirmExit(false);
                    }}
                  >
                    留在设置
                  </Button>
                  <Button
                    type="button"
                    onClick={() => {
                      finishExit();
                    }}
                  >
                    确认关闭
                  </Button>
                </div>
              </div>
            </div>
          ) : null}
        </div>
      </DialogContent>
    </Dialog>
  );
}

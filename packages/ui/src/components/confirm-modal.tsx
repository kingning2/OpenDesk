/**
 * ConfirmModal — 通用确认弹窗（替代原生 confirm）。
 *
 * 基于现有 Dialog 原语，沿用 Emil 动效；支持 warning/danger/info 类型与 loading。
 * 从原前端 `components/common/ConfirmModal.tsx` 抽取为公共组件。
 */

import { AlertTriangle, Info, Loader2 } from "lucide-react";
import * as React from "react";

import { cn } from "../lib/cn";
import { Button } from "./button";
import { Dialog, DialogContent, DialogTitle } from "./dialog";

export interface ConfirmModalProps {
  isOpen: boolean;
  title?: string;
  message: React.ReactNode;
  confirmText?: string;
  cancelText?: string;
  type?: "warning" | "danger" | "info";
  loading?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

const iconMap = {
  warning: <AlertTriangle className="size-6 text-amber-500" aria-hidden />,
  danger: <AlertTriangle className="size-6 text-red-500" aria-hidden />,
  info: <Info className="size-6 text-blue-500" aria-hidden />,
} as const;

/**
 * 通用确认弹窗。
 *
 * @author agent
 * @created 2026-08-13
 *
 * @param props - 见 {@link ConfirmModalProps}
 * @returns 确认弹窗节点
 */
export function ConfirmModal({
  isOpen,
  title = "确认操作",
  message,
  confirmText = "确定",
  cancelText = "取消",
  type = "warning",
  loading = false,
  onConfirm,
  onCancel,
}: ConfirmModalProps) {
  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onCancel()}>
      <DialogContent
        dismissOnOutsidePress={false}
        className="max-w-sm text-center"
        title={title}
        footer={
          <div className="flex w-full gap-3">
            <Button variant="secondary" className="flex-1" onClick={onCancel} disabled={loading}>
              {cancelText}
            </Button>
            <Button
              className={cn(
                "flex-1",
                type === "danger" && "bg-destructive text-destructive-foreground hover:bg-destructive/90",
                type === "warning" && "bg-amber-500 text-white hover:bg-amber-600",
              )}
              onClick={onConfirm}
              disabled={loading}
            >
              {loading ? <Loader2 className="size-4 animate-spin" aria-hidden /> : null}
              {confirmText}
            </Button>
          </div>
        }
      >
        <DialogTitle className="flex items-center justify-center gap-2">
          {iconMap[type]}
        </DialogTitle>
        <p className="text-[length:var(--text-sm)] leading-relaxed text-muted-foreground">
          {message}
        </p>
      </DialogContent>
    </Dialog>
  );
}

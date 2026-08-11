/**
 * 扫码登录弹窗 — 显示二维码 + 轮询扫码状态。
 */

import { useEffect, useRef, useState } from "react";
import { Button, Dialog, DialogContent } from "@desk/ui";
import {
  channelQrCancel,
  channelQrCheck,
  channelQrStart,
} from "@desk/platform/ipc/channel";

type QrStatus =
  | "ready"
  | "waiting"
  | "scanned"
  | "confirmed"
  | "success"
  | "expired"
  | "failed"
  | "error";

const POLL_INTERVAL_MS = 2000;

/**
 * 扫码登录弹窗。
 */
export function QrLoginDialog({
  open,
  accountId,
  onClose,
  onSuccess,
}: {
  open: boolean;
  accountId: string;
  onClose: () => void;
  onSuccess: () => void;
}) {
  const [status, setStatus] = useState<QrStatus>("ready");
  const [qrBase64, setQrBase64] = useState<string | null>(null);
  const [message, setMessage] = useState("正在生成二维码…");
  const sessionRef = useRef<string | null>(null);
  const cancelledRef = useRef(false);

  // 启动扫码登录（对话框由父级 key 重挂载保证初始状态）。
  useEffect(() => {
    if (!open) {
      return;
    }
    cancelledRef.current = false;

    void channelQrStart({ account_id: accountId })
      .then((result) => {
        if (cancelledRef.current) {
          return;
        }
        if (!result.ok || !result.qr_base64) {
          setStatus("failed");
          setMessage(result.detail ?? "二维码生成失败");
          return;
        }
        sessionRef.current = result.session_id ?? null;
        setQrBase64(result.qr_base64);
        setStatus("waiting");
        setMessage("请用闲鱼 App 扫码");
      })
      .catch((error) => {
        if (!cancelledRef.current) {
          setStatus("failed");
          setMessage(error instanceof Error ? error.message : String(error));
        }
      });

    return () => {
      cancelledRef.current = true;
    };
  }, [open, accountId]);

  // 轮询扫码状态。
  useEffect(() => {
    if (!open || !qrBase64 || status === "success") {
      return;
    }
    const timer = window.setInterval(async () => {
      if (!sessionRef.current) {
        return;
      }
      try {
        const result = await channelQrCheck({ session_id: sessionRef.current });
        if (cancelledRef.current) {
          return;
        }
        const newStatus = result.status as QrStatus;
        setStatus(newStatus);
        switch (newStatus) {
          case "waiting":
            setMessage("请用闲鱼 App 扫码");
            break;
          case "scanned":
          case "confirmed":
            setMessage("已扫码，请在手机确认登录");
            break;
          case "success":
            setMessage("登录成功！");
            window.clearInterval(timer);
            onSuccess();
            onClose();
            break;
          case "expired":
            setMessage("二维码已过期，请重新打开");
            window.clearInterval(timer);
            break;
          case "failed":
            setMessage(result.detail ?? "登录失败");
            window.clearInterval(timer);
            break;
          default:
            setMessage("等待扫码…");
        }
      } catch (error) {
        if (!cancelledRef.current) {
          setMessage(error instanceof Error ? error.message : String(error));
        }
      }
    }, POLL_INTERVAL_MS);

    return () => window.clearInterval(timer);
  }, [open, qrBase64, status, onSuccess, onClose]);

  // 取消扫码登录。
  async function handleCancel() {
    cancelledRef.current = true;
    if (sessionRef.current) {
      try {
        await channelQrCancel({ session_id: sessionRef.current });
      } catch {
        // 忽略取消错误。
      }
    }
    onClose();
  }

  const isTerminal =
    status === "success" || status === "expired" || status === "failed";

  return (
    <Dialog open={open} onOpenChange={(next) => {
      if (!next && !isTerminal) {
        void handleCancel();
      }
    }}>
      <DialogContent className="w-[340px] max-w-[90vw]">
        <div className="flex flex-col items-center gap-4 p-4">
          <h3 className="text-[length:var(--text-lg)] font-semibold tracking-tight">扫码登录</h3>

          {!qrBase64 && status === "ready" ? (
            <p className="py-10 text-[length:var(--text-sm)] text-muted-foreground">正在生成二维码…</p>
          ) : qrBase64 ? (
            <img
              src={qrBase64}
              alt="登录二维码"
              className="size-56 rounded-[var(--radius-lg)] border border-border object-contain"
            />
          ) : (
            <div className="flex size-56 items-center justify-center rounded-[var(--radius-lg)] border border-dashed border-border">
              <p className="px-4 text-center text-[length:var(--text-sm)] text-muted-foreground">
                {message}
              </p>
            </div>
          )}

          <p
            className={`text-center text-[length:var(--text-sm)] ${
              status === "scanned" || status === "confirmed"
                ? "text-amber-600"
                : status === "failed" || status === "expired"
                  ? "text-destructive"
                  : status === "success"
                    ? "text-emerald-600"
                    : "text-muted-foreground"
            }`}
          >
            {message}
          </p>

          <div className="flex w-full justify-center gap-2">
            {isTerminal ? (
              <Button onClick={onClose}>关闭</Button>
            ) : (
              <Button variant="ghost" onClick={() => void handleCancel()}>取消</Button>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

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
import { useLogStore } from "@feature/log";

type QrStatus =
  | "ready"
  | "waiting"
  | "scanned"
  | "confirmed"
  | "success"
  | "refreshed"
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
  name,
  kind,
  onClose,
  onSuccess,
}: {
  open: boolean;
  accountId: string;
  /** 自动创建账号时的名称（无账号场景）。 */
  name?: string;
  /** 自动创建账号时的平台类型。 */
  kind?: string;
  onClose: () => void;
  onSuccess: () => void;
}) {
  const [status, setStatus] = useState<QrStatus>("ready");
  const [qrBase64, setQrBase64] = useState<string | null>(null);
  const [message, setMessage] = useState("正在生成二维码…");
  const sessionRef = useRef<string | null>(null);
  const cancelledRef = useRef(false);
  // StrictMode 下 effect 会连续挂载两次；标记只允许启动一次扫码会话，
  // 否则会并发打开两个 Playwright 浏览器，二维码迟迟不返回。
  const startedRef = useRef(false);

  /**
   * 开发环境：把二维码推进日志面板渲染成图（生产不推，避免日志膨胀）。
   *
   * @author Xiaoman
   * @created 2026-08-13
   * @param qr - data URL 二维码
   */
  function pushQrLog(qr: string) {
    if (!import.meta.env.DEV) {
      return;
    }
    useLogStore.getState().append([
      { ts: Date.now(), level: "INFO", source: "react", target: "qr", message: qr },
    ]);
  }

  // 启动扫码登录（对话框由父级 key 重挂载保证初始状态）。
  useEffect(() => {
    if (!open) {
      return;
    }
    if (startedRef.current) {
      // 第二次挂载：仅恢复取消标记，不重复启动。
      cancelledRef.current = false;
      return () => {
        cancelledRef.current = true;
      };
    }
    startedRef.current = true;
    cancelledRef.current = false;

    void channelQrStart({ account_id: accountId, name, kind })
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
        pushQrLog(result.qr_base64);
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
      // 关闭弹窗时取消后端扫码会话，及时释放浏览器。
      const sessionId = sessionRef.current;
      if (sessionId) {
        void channelQrCancel({ session_id: sessionId }).catch(() => {});
      }
    };
    // kind/name 有意不进依赖：名称变化不应重启已开始的扫码会话。
  }, [open, accountId]); // eslint-disable-line react-hooks/exhaustive-deps

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
          case "refreshed":
            // 二维码已过期，侧车原地换新码：直接换图，继续轮询。
            if (result.qr_base64) {
              setQrBase64(result.qr_base64);
              pushQrLog(result.qr_base64);
            }
            setStatus("waiting");
            setMessage("二维码已刷新，请扫码");
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

          {/* 二维码未就绪时文案在占位框内展示，避免重复。 */}
          {qrBase64 ? (
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
          ) : null}

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

/**
 * 闲鱼账号管理页（迁移自原前端 `pages/accounts/Accounts.tsx`）。
 *
 * 账号列表 + 筛选 + 扫码登录添加 + 状态切换 + 连接管理。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/account`），复用 crates/app 账号服务。
 */

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  Button,
  Checkbox,
  ConfirmModal,
  Dialog,
  DialogContent,
  Input,
  Loading,
  PageCardGrid,
  PageGlowCard,
  PageScaffold,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Textarea,
  toast,
} from "@desk/ui";
import { QrCode, Trash2 } from "@desk/ui/icons";
import {
  accountConnect,
  accountConnectionState,
  accountDelete,
  accountDisconnect,
  accountList,
  accountQrCancel,
  accountQrCheck,
  accountQrStart,
  accountSetStatus,
  accountUpdate,
  type AccountStatus,
  type XianyuAccount,
} from "@desk/platform/ipc/account";
import { listenChannelStatus } from "@desk/platform/events";
import {
  CHANNEL_CONNECTION_STATUS_MAP,
  connectionStatusHint,
  mergeChannelConnectionState,
  normalizeChannelConnectionState,
  type ChannelConnectionState,
} from "@desk/platform";
import {
  loadAutoConnectConfig,
  runAutoConnectNow,
  setAccountAutoConnect,
  setAutoConnectOnStartEnabled,
} from "./use-auto-connect";

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入

/**
 * IPC / toast 错误是否为登录过期（连接失败时写入 auth_expired）。
 *
 * @author Xiaoman
 * @created 2026-08-21
 */
function isAuthExpiredText(text?: string | null): boolean {
  if (!text) {
    return false;
  }
  return (
    text.includes("FAIL_SYS_SESSION_EXPIRED") ||
    text.includes("Session过期") ||
    text.includes("SESSION_EXPIRED") ||
    text.includes("登录态已过期") ||
    text.includes("请重新扫码登录") ||
    text.includes("Cookie 无效") ||
    text.includes("cookie 缺少")
  );
}

type QrStatus = "ready" | "waiting" | "scanned" | "confirmed" | "success" | "expired" | "failed";

/**
 * 扫码登录弹窗：显示二维码 + 轮询状态，成功后自动创建或更新账号 Cookie。
 *
 * @author Xiaoman
 * @created 2026-08-13
 */
function AccountQrDialog({
  open,
  onClose,
  onSuccess,
  title = "扫码登录",
  hint = "请用闲鱼 App 扫码",
}: {
  open: boolean;
  onClose: () => void;
  onSuccess: () => void;
  /** 弹窗标题（重新登录时可改为「重新扫码登录」）。 */
  title?: string;
  /** 等待扫码时的提示文案。 */
  hint?: string;
}) {
  const [status, setStatus] = useState<QrStatus>("ready");
  const [qrBase64, setQrBase64] = useState<string | null>(null);
  const [message, setMessage] = useState("正在生成二维码…");
  const sessionRef = useRef<string | null>(null);
  const cancelledRef = useRef(false);
  const startedRef = useRef(false);

  // 启动扫码（key 重挂载保证初始状态）。
  useEffect(() => {
    if (!open) {
      return;
    }
    if (startedRef.current) {
      cancelledRef.current = false;
      return () => {
        cancelledRef.current = true;
      };
    }
    startedRef.current = true;
    cancelledRef.current = false;

    void accountQrStart()
      .then((result) => {
        if (cancelledRef.current) {
          return;
        }
        if (!result.ok || !result.qr_base64) {
          setStatus("failed");
          setMessage(result.detail ?? "二维码生成失败");
          return;
        }
        sessionRef.current = result.session_id;
        setQrBase64(result.qr_base64);
        setStatus("waiting");
        setMessage(hint);
      })
      .catch((error) => {
        if (!cancelledRef.current) {
          setStatus("failed");
          setMessage(error instanceof Error ? error.message : String(error));
        }
      });

    return () => {
      cancelledRef.current = true;
      const sessionId = sessionRef.current;
      if (sessionId) {
        void accountQrCancel(sessionId).catch(() => {});
      }
    };
  }, [open, hint]);

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
        const result = await accountQrCheck(sessionRef.current);
        if (cancelledRef.current) {
          return;
        }
        const nextStatus = result.status as QrStatus;
        setStatus(nextStatus);
        switch (nextStatus) {
          case "waiting":
            setMessage(hint);
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
    }, 2000);

    return () => window.clearInterval(timer);
  }, [open, qrBase64, status, onSuccess, onClose, hint]);

  const isTerminal = status === "success" || status === "expired" || status === "failed";

  async function handleCancel() {
    cancelledRef.current = true;
    if (sessionRef.current) {
      try {
        await accountQrCancel(sessionRef.current);
      } catch {
        // 忽略取消错误。
      }
    }
    onClose();
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        if (!next && !isTerminal) {
          void handleCancel();
        }
      }}
    >
      <DialogContent className="w-[340px] max-w-[90vw]">
        <div className="flex flex-col items-center gap-4 p-4">
          <h3 className="text-[length:var(--text-lg)] font-semibold tracking-tight">{title}</h3>

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

/**
 * 闲鱼账号管理页。
 *
 * @author Xiaoman
 * @created 2026-08-13
 */
export function XianyuAccountsPage() {
  const [accounts, setAccounts] = useState<XianyuAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [keyword, setKeyword] = useState("");
  const [statusFilter, setStatusFilter] = useState<string>("all");
  const [deleteTarget, setDeleteTarget] = useState<XianyuAccount | null>(null);
  const [qrOpen, setQrOpen] = useState(false);
  const [qrSeq, setQrSeq] = useState(0);
  /** 重新扫码时的弹窗文案；普通添加账号时用默认。 */
  const [qrTitle, setQrTitle] = useState("扫码登录");
  const [qrHint, setQrHint] = useState("请用闲鱼 App 扫码");
  const [editorOpen, setEditorOpen] = useState(false);
  const [editingAccount, setEditingAccount] = useState<XianyuAccount | null>(null);
  const [editorDisplayName, setEditorDisplayName] = useState("");
  const [editorRemark, setEditorRemark] = useState("");
  const [editorCookie, setEditorCookie] = useState("");
  const [editorSaving, setEditorSaving] = useState(false);
  /** account_id → 渠道连接状态（与 `channel/status.state` / map 对齐）。 */
  const [connectionStates, setConnectionStates] = useState<
    Record<string, ChannelConnectionState>
  >({});
  /** account_id → 后端短文案 hint（禁止原始 JSON）。 */
  const [connectionDetails, setConnectionDetails] = useState<Record<string, string>>({});
  const [connectingId, setConnectingId] = useState<string | null>(null);
  const [autoConnectOnStart, setAutoConnectOnStart] = useState(false);
  const [autoConnectIds, setAutoConnectIds] = useState<string[]>([]);
  const [autoConnecting, setAutoConnecting] = useState(false);

  async function load() {
    setLoading(true);
    try {
      const list = await accountList(OWNER_ID);
      setAccounts(list);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    let cancelled = false;
    void accountList(OWNER_ID)
      .then((list) => {
        if (!cancelled) {
          setAccounts(list);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          toast.error(error instanceof Error ? error.message : String(error));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    let cancelled = false;
    void loadAutoConnectConfig()
      .then((config) => {
        if (!cancelled) {
          setAutoConnectOnStart(config.enabled);
          setAutoConnectIds(config.accountIds);
        }
      })
      .catch(() => {
        // 读取失败时保持默认关闭，不阻断账号列表。
      });
    return () => {
      cancelled = true;
    };
  }, []);

  /** 一次性拉取连接状态快照（进页 / 扫码成功后）；运行中靠事件推送。 */
  const refreshConnectionStates = useCallback(async (accountIds: string[]) => {
    if (accountIds.length === 0) {
      return;
    }
    const updates: Record<string, ChannelConnectionState> = {};
    await Promise.all(
      accountIds.map(async (accountId) => {
        try {
          updates[accountId] = normalizeChannelConnectionState(
            await accountConnectionState(OWNER_ID, accountId),
          );
        } catch {
          // 单账号状态查询失败不阻断其余。
        }
      }),
    );
    if (Object.keys(updates).length === 0) {
      return;
    }
    // 登录过期 / 过滑块中会伴随 disconnect，快照不得冲掉合成态。
    setConnectionStates((current) => {
      const merged = { ...current };
      for (const [accountId, incoming] of Object.entries(updates)) {
        if (
          (current[accountId] === "auth_expired" ||
            current[accountId] === "renewing" ||
            current[accountId] === "queued") &&
          incoming !== "connected"
        ) {
          continue;
        }
        merged[accountId] = incoming;
      }
      return merged;
    });
  }, []);

  // 订阅 Rust 侧 channel/status：只信 canonical `state`，用 map 渲染。
  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    void listenChannelStatus((payload) => {
      if (cancelled || !payload.account_id) {
        return;
      }
      setConnectionStates((current) => ({
        ...current,
        [payload.account_id]: mergeChannelConnectionState(
          current[payload.account_id],
          payload.state,
        ),
      }));
      setConnectionDetails((current) => {
        if (payload.state === "connected") {
          const next = { ...current };
          delete next[payload.account_id];
          return next;
        }
        const hint = connectionStatusHint(
          normalizeChannelConnectionState(payload.state),
          payload.detail,
        );
        if (hint) {
          return { ...current, [payload.account_id]: hint };
        }
        return current;
      });
      if (payload.state === "connected") {
        void load();
      }
    })
      .then((fn) => {
        if (cancelled) {
          fn();
          return;
        }
        unlisten = fn;
      })
      .catch(() => {
        // 事件订阅失败时仍依赖进页 IPC 快照。
      });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  // 进页 / 账号列表变化时拉一次快照；运行中只信 channel/status 推送，不再定时轮询。
  useEffect(() => {
    if (accounts.length === 0) {
      return;
    }
    void refreshConnectionStates(accounts.map((account) => account.account_id));
  }, [accounts, refreshConnectionStates]);

  const filtered = useMemo(() => {
    return accounts.filter((account) => {
      const matchKeyword =
        !keyword.trim() ||
        account.account_id.includes(keyword.trim()) ||
        account.display_name.includes(keyword.trim());
      const matchStatus =
        statusFilter === "all" || (statusFilter === "active" ? account.status === "active" : account.status === "disabled");
      return matchKeyword && matchStatus;
    });
  }, [accounts, keyword, statusFilter]);

  async function handleToggleStatus(account: XianyuAccount) {
    const next: AccountStatus = account.status === "active" ? "disabled" : "active";
    try {
      await accountSetStatus(OWNER_ID, account.account_id, next);
      toast.success(`账号已${next === "active" ? "启用" : "停用"}`);
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  function openAccountEditor(account: XianyuAccount) {
    setEditingAccount(account);
    setEditorDisplayName(account.display_name);
    setEditorRemark(account.remark);
    setEditorCookie(account.cookie ?? "");
    setEditorOpen(true);
  }

  async function handleSaveAccountProfile() {
    if (!editingAccount) {
      return;
    }
    try {
      setEditorSaving(true);
      const cookieChanged = editorCookie.trim() !== (editingAccount.cookie ?? "");
      await accountUpdate(OWNER_ID, editingAccount.account_id, {
        display_name: editorDisplayName.trim(),
        remark: editorRemark.trim(),
        cookie: editorCookie.trim() || undefined,
      });
      toast.success(cookieChanged ? "Cookie 已更新，请先断开再连接以生效" : "账号配置已更新");
      setEditorOpen(false);
      setEditingAccount(null);
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setEditorSaving(false);
    }
  }

  async function handleConnect(account: XianyuAccount) {
    setConnectingId(account.account_id);
    try {
      const state = normalizeChannelConnectionState(
        await accountConnect(OWNER_ID, account.account_id),
      );
      setConnectionStates((current) => ({ ...current, [account.account_id]: state }));
      setConnectionDetails((current) => {
        const next = { ...current };
        delete next[account.account_id];
        return next;
      });
      await load();
      toast.success(
        state === "connected" ? "连接成功，已同步用户资料并开始监听消息" : `连接状态：${CHANNEL_CONNECTION_STATUS_MAP[state].label}`,
      );
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      if (isAuthExpiredText(message)) {
        setConnectionStates((current) => ({
          ...current,
          [account.account_id]: "auth_expired",
        }));
        setConnectionDetails((current) => ({
          ...current,
          [account.account_id]: "登录态已过期，请重新扫码后再连接",
        }));
        toast.error("登录态已过期，请重新扫码后再连接", {
          action: {
            label: "重新扫码",
            onClick: () => openRescanQr(account),
          },
        });
      } else {
        toast.error(message);
      }
    } finally {
      setConnectingId(null);
    }
  }

  async function handleDisconnect(account: XianyuAccount) {
    try {
      await accountDisconnect(OWNER_ID, account.account_id);
      setConnectionStates((current) => ({ ...current, [account.account_id]: "disconnected" }));
      toast.success("已断开连接");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function handleDelete() {
    if (!deleteTarget) return;
    try {
      await accountDelete(OWNER_ID, deleteTarget.account_id);
      setAutoConnectIds(await setAccountAutoConnect(deleteTarget.account_id, false));
      toast.success("账号已删除");
      setDeleteTarget(null);
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function handleToggleAutoConnect() {
    const next = !autoConnectOnStart;
    try {
      await setAutoConnectOnStartEnabled(next);
      setAutoConnectOnStart(next);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
      return;
    }
    if (!next) {
      toast.success("已关闭启动自动连接");
      return;
    }
    if (autoConnectIds.length === 0) {
      toast.success("已开启启动自动连接：请在账号卡片上勾选要自动连接的账号");
      return;
    }
    toast.success("已开启：启动时只连接勾选账号；风控未过滑块将 10 分钟后重试");
    setAutoConnecting(true);
    try {
      const count = await runAutoConnectNow();
      toast.success(
        count > 0 ? `已开始连接 ${count} 个勾选账号` : "勾选账号中暂无可连接项（需启用且有 Cookie）",
      );
      await refreshConnectionStates(accounts.map((account) => account.account_id));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setAutoConnecting(false);
    }
  }

  async function handleToggleAccountAutoConnect(accountId: string, selected: boolean) {
    try {
      const next = await setAccountAutoConnect(accountId, selected);
      setAutoConnectIds(next);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  function openQrLogin() {
    setQrTitle("扫码登录");
    setQrHint("请用闲鱼 App 扫码");
    setQrSeq((seq) => seq + 1);
    setQrOpen(true);
  }

  /**
   * 打开重新扫码弹窗（同 unb 账号会更新 Cookie，无需新建）。
   *
   * @author Xiaoman
   * @created 2026-08-21
   *
   * @param account - 需要刷新登录态的账号
   */
  function openRescanQr(account: XianyuAccount) {
    const name = account.display_name || account.account_id;
    setQrTitle("重新扫码登录");
    setQrHint(`请用闲鱼 App 扫码，刷新「${name}」的登录态`);
    setQrSeq((seq) => seq + 1);
    setQrOpen(true);
  }

  return (
    <PageScaffold
      title="账号管理"
      subtitle="勾选账号后开启启动自动连接；未勾选的不会自动连"
      extra={
        <div className="flex flex-wrap items-center gap-2">
          <Button
            variant={autoConnectOnStart ? "default" : "outline"}
            disabled={autoConnecting}
            onClick={() => void handleToggleAutoConnect()}
            title="开启后启动应用只连接卡片上勾选的账号；触发风控且滑块未过时，约 10 分钟后自动再连"
          >
            {autoConnecting
              ? "连接中…"
              : autoConnectOnStart
                ? `启动自动连接：开（${autoConnectIds.length}）`
                : "启动自动连接：关"}
          </Button>
          <Button variant="outline" onClick={openQrLogin}>
            <QrCode className="size-4" aria-hidden />
            扫码登录
          </Button>
        </div>
      }
      toolbar={
        <div className="flex flex-wrap items-center gap-3">
          <Input
            placeholder="搜索账号 ID / 名称"
            value={keyword}
            onChange={(event) => setKeyword(event.target.value)}
            className="w-56"
          />
          <Select value={statusFilter} onValueChange={setStatusFilter}>
            <SelectTrigger className="w-32">
              <SelectValue placeholder="全部状态" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部状态</SelectItem>
              <SelectItem value="active">已启用</SelectItem>
              <SelectItem value="disabled">已停用</SelectItem>
            </SelectContent>
          </Select>
        </div>
      }
    >
      {loading ? (
        <Loading size="lg" text="加载中..." className="py-16" />
      ) : filtered.length === 0 ? (
        <div className="flex flex-col items-center gap-4 py-16 text-center">
          <p className="text-muted-foreground">暂无账号，请先扫码登录添加</p>
          <Button variant="outline" onClick={openQrLogin}>
            <QrCode className="size-4" aria-hidden />
            扫码登录
          </Button>
        </div>
      ) : (
        <PageCardGrid>
            {filtered.map((account) => {
              const displayState = normalizeChannelConnectionState(
                connectionStates[account.account_id],
              );
              const conn = CHANNEL_CONNECTION_STATUS_MAP[displayState];
              const hint = connectionStatusHint(
                displayState,
                connectionDetails[account.account_id],
              );
              const authExpired = displayState === "auth_expired";
              const renewing = displayState === "renewing";
              const queued = displayState === "queued";
              const sliderBusy = renewing || queued;
              const isConnecting = connectingId === account.account_id;
              return (
                <PageGlowCard
                  key={account.account_id}
                  role="button"
                  tabIndex={0}
                  className="flex h-full flex-col text-left transition hover:bg-muted/10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  onClick={() => openAccountEditor(account)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter" || event.key === " ") {
                      event.preventDefault();
                      openAccountEditor(account);
                    }
                  }}
                >
                  <div className="relative flex h-full min-h-0 flex-col rounded-[inherit] border border-border bg-card p-4 transition hover:border-primary/40">
                  <div className="flex items-start justify-between gap-3">
                    <div className="flex min-w-0 items-start gap-3">
                      {account.avatar_url ? (
                        <img
                          src={account.avatar_url}
                          alt=""
                          className="size-10 shrink-0 rounded-full border border-border object-cover"
                        />
                      ) : (
                        <div
                          className="flex size-10 shrink-0 items-center justify-center rounded-full border border-border bg-muted text-[length:var(--text-sm)] font-medium text-muted-foreground"
                          aria-hidden
                        >
                          {(account.display_name || account.account_id).slice(0, 1)}
                        </div>
                      )}
                      <div className="min-w-0 space-y-1">
                        <div className="truncate text-[length:var(--text-base)] font-medium">
                          {account.display_name || account.account_id}
                        </div>
                        <div className="truncate font-mono text-[length:var(--text-xs)] text-muted-foreground">
                          {account.account_id}
                        </div>
                      </div>
                    </div>
                    <span
                      className={
                        account.status === "active"
                          ? "rounded-full bg-emerald-500/15 px-2 py-0.5 text-[length:var(--text-xs)] text-emerald-500"
                          : "rounded-full bg-muted px-2 py-0.5 text-[length:var(--text-xs)] text-muted-foreground"
                      }
                    >
                      {account.status === "active" ? "启用" : "停用"}
                    </span>
                  </div>

                  <div className="mt-4 min-h-0 flex-1 space-y-2 text-[length:var(--text-sm)]">
                    <div className="flex items-center justify-between gap-3">
                      <span className="text-muted-foreground">连接状态</span>
                      <span
                        className={`rounded-full px-2 py-0.5 text-[length:var(--text-xs)] ${conn.badgeClass}`}
                        title={hint ?? undefined}
                      >
                        {conn.label}
                      </span>
                    </div>
                    {/* 固定占位，避免有无提示文案时同行高度视觉跳动 */}
                    <p
                      className={`min-h-[1.25rem] text-[length:var(--text-xs)] ${
                        authExpired
                          ? "text-orange-700"
                          : renewing
                            ? "text-sky-700"
                            : queued
                              ? "text-violet-700"
                              : displayState === "error"
                                ? "text-red-600"
                                : "invisible"
                      }`}
                      aria-hidden={!hint}
                    >
                      {hint ?? "占位"}
                    </p>
                    <div className="flex items-start justify-between gap-3">
                      <span className="text-muted-foreground">备注</span>
                      <span className="line-clamp-2 max-w-[65%] text-right">{account.remark || "—"}</span>
                    </div>
                  </div>

                  <div
                    className="mt-4 flex flex-wrap items-center gap-2"
                    onClick={(event) => event.stopPropagation()}
                  >
                    <label className="mr-auto flex items-center gap-2 text-[length:var(--text-sm)] text-muted-foreground">
                      <Checkbox
                        checked={autoConnectIds.includes(account.account_id)}
                        onCheckedChange={(checked) =>
                          void handleToggleAccountAutoConnect(account.account_id, checked === true)
                        }
                        aria-label={`自动连接 ${account.display_name || account.account_id}`}
                      />
                      自动连接
                    </label>
                    {authExpired ? (
                      <Button size="sm" onClick={() => openRescanQr(account)}>
                        <QrCode className="size-3.5" aria-hidden />
                        重新扫码
                      </Button>
                    ) : null}
                    {connectionStates[account.account_id] === "connected" ? (
                      <Button size="sm" variant="outline" onClick={() => void handleDisconnect(account)}>
                        断开
                      </Button>
                    ) : (
                      <Button
                        size="sm"
                        variant="outline"
                        disabled={isConnecting || !account.cookie || authExpired || sliderBusy}
                        onClick={() => void handleConnect(account)}
                        title={
                          authExpired
                            ? "请先重新扫码刷新登录态"
                            : renewing
                              ? "正在过滑块，请稍候"
                              : queued
                                ? "排队等待过滑块，请稍候"
                                : undefined
                        }
                      >
                        {isConnecting ? "连接中…" : "连接"}
                      </Button>
                    )}
                    <Button size="sm" variant="outline" onClick={() => void handleToggleStatus(account)}>
                      {account.status === "active" ? "停用" : "启用"}
                    </Button>
                    <Button size="sm" variant="ghost" onClick={() => openAccountEditor(account)}>
                      编辑资料
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      className="text-destructive"
                      onClick={() => setDeleteTarget(account)}
                    >
                      <Trash2 className="size-3.5" aria-hidden />
                    </Button>
                  </div>
                  </div>
                </PageGlowCard>
              );
            })}
          </PageCardGrid>
      )}

      {/* 扫码登录弹窗 */}
      <AccountQrDialog
        key={qrSeq}
        open={qrOpen}
        title={qrTitle}
        hint={qrHint}
        onClose={() => setQrOpen(false)}
        onSuccess={() => {
          void (async () => {
            const list = await accountList(OWNER_ID);
            setAccounts(list);
            await refreshConnectionStates(list.map((account) => account.account_id));
            toast.success("扫码成功，登录态已刷新");
          })().catch((error) => {
            toast.error(error instanceof Error ? error.message : String(error));
          });
        }}
      />

      <ConfirmModal
        isOpen={editorOpen}
        title={editingAccount ? `编辑账号 · ${editingAccount.account_id}` : "编辑账号"}
        message={
          <div className="space-y-3 text-left">
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">显示名称</span>
              <Input
                value={editorDisplayName}
                onChange={(event) => setEditorDisplayName(event.target.value)}
                placeholder="账号展示名"
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">备注</span>
              <Input
                value={editorRemark}
                onChange={(event) => setEditorRemark(event.target.value)}
                placeholder="可选备注"
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">
                Cookie（风控/登录后更新）
              </span>
              <Textarea
                value={editorCookie}
                onChange={(event) => setEditorCookie(event.target.value)}
                placeholder="在真实浏览器登录闲鱼并过滑块后，复制 Cookie 粘贴这里"
                rows={5}
                className="font-mono text-xs"
              />
              <span className="block text-[length:var(--text-xs)] text-muted-foreground/80">
                在浏览器控制台输入 document.cookie 复制，或 F12 → Application → Cookies。留空则不修改 Cookie。
              </span>
            </label>
          </div>
        }
        confirmText={editorSaving ? "保存中…" : "保存"}
        loading={editorSaving}
        onConfirm={() => void handleSaveAccountProfile()}
        onCancel={() => {
          if (editorSaving) {
            return;
          }
          setEditorOpen(false);
          setEditingAccount(null);
        }}
      />

      {/* 删除确认 */}
      <ConfirmModal
        isOpen={deleteTarget !== null}
        type="danger"
        title="删除账号"
        message={`确认删除账号「${deleteTarget?.account_id ?? ""}」？该操作不可撤销。`}
        confirmText="删除"
        onConfirm={() => void handleDelete()}
        onCancel={() => setDeleteTarget(null)}
      />
    </PageScaffold>
  );
}

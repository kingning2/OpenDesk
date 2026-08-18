/**
 * 闲鱼账号管理页（迁移自原前端 `pages/accounts/Accounts.tsx`）。
 *
 * 按原前端核心交互重写：账号列表 + 筛选 + 新建/编辑 + 状态切换 + 扫码登录。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/account`），复用 crates/app 账号服务。
 */

import { useEffect, useMemo, useRef, useState } from "react";
import {
  Button,
  ConfirmModal,
  Dialog,
  DialogContent,
  Input,
  Loading,
  PageScaffold,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  toast,
} from "@desk/ui";
import { Plus, QrCode, Trash2 } from "@desk/ui/icons";
import {
  accountConnect,
  accountConnectionState,
  accountCreate,
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

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入

/** 连接状态标签。 */
const CONNECTION_LABELS: Record<string, { label: string; cls: string }> = {
  connected: { label: "已连接", cls: "bg-emerald-500/15 text-emerald-600" },
  connecting: { label: "连接中", cls: "bg-amber-500/15 text-amber-600" },
  disconnected: { label: "未连接", cls: "bg-muted text-muted-foreground" },
  error: { label: "异常", cls: "bg-red-500/15 text-red-600" },
};

type QrStatus = "ready" | "waiting" | "scanned" | "confirmed" | "success" | "expired" | "failed";

/** 扫码登录弹窗：显示二维码 + 轮询状态，成功后自动创建账号。 */
function AccountQrDialog({
  open,
  onClose,
  onSuccess,
}: {
  open: boolean;
  onClose: () => void;
  onSuccess: () => void;
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
      const sessionId = sessionRef.current;
      if (sessionId) {
        void accountQrCancel(sessionId).catch(() => {});
      }
    };
  }, [open]);

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
    }, 2000);

    return () => window.clearInterval(timer);
  }, [open, qrBase64, status, onSuccess, onClose]);

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

          <p className="text-center text-[length:var(--text-xs)] text-muted-foreground">
            提示：此处的扫码创建的是业务账号（用于自动回复管理）。
            <br />
            实时接收闲鱼消息请在侧栏「会话工作台」中扫码并连接。
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

interface AccountForm {
  account_id: string;
  display_name: string;
  cookie: string;
  remark: string;
}

const EMPTY_FORM: AccountForm = { account_id: "", display_name: "", cookie: "", remark: "" };

function toAccount(ownerId: number, form: AccountForm): XianyuAccount {
  return {
    id: 0,
    owner_id: ownerId,
    account_id: form.account_id.trim(),
    display_name: form.display_name.trim(),
    unb: "",
    cookie: form.cookie,
    login_method: "qr",
    status: "active",
    remark: form.remark.trim(),
    pause_duration_minutes: 10,
  };
}

/**
 * 闲鱼账号管理页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuAccountsPage() {
  const [accounts, setAccounts] = useState<XianyuAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [keyword, setKeyword] = useState("");
  const [statusFilter, setStatusFilter] = useState<string>("all");
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState<AccountForm>(EMPTY_FORM);
  const [saving, setSaving] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<XianyuAccount | null>(null);
  const [qrOpen, setQrOpen] = useState(false);
  const [qrSeq, setQrSeq] = useState(0);
  /** account_id → 渠道连接状态。 */
  const [connectionStates, setConnectionStates] = useState<Record<string, string>>({});
  const [connectingId, setConnectingId] = useState<string | null>(null);

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

  // 轮询各账号的渠道连接状态（每 5 秒）。
  useEffect(() => {
    let cancelled = false;
    const timer = window.setInterval(async () => {
      if (cancelled) return;
      const states: Record<string, string> = {};
      for (const account of accounts) {
        try {
          states[account.account_id] = await accountConnectionState(OWNER_ID, account.account_id);
        } catch {
          // 单账号状态查询失败不阻断其余。
        }
      }
      if (!cancelled) {
        setConnectionStates(states);
      }
    }, 5000);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [accounts]);

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

  async function handleCreate() {
    if (!form.account_id.trim() || !form.cookie.trim()) {
      toast.error("账号标识与 Cookie 不能为空");
      return;
    }
    setSaving(true);
    try {
      await accountCreate(OWNER_ID, toAccount(OWNER_ID, form));
      toast.success("账号创建成功");
      setShowForm(false);
      setForm(EMPTY_FORM);
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

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

  async function handleUpdateRemark(account: XianyuAccount, remark: string) {
    try {
      await accountUpdate(OWNER_ID, account.account_id, { remark });
      toast.success("备注已更新");
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function handleConnect(account: XianyuAccount) {
    setConnectingId(account.account_id);
    try {
      const state = await accountConnect(OWNER_ID, account.account_id);
      setConnectionStates((current) => ({ ...current, [account.account_id]: state }));
      toast.success(state === "connected" ? "连接成功，开始监听闲鱼消息" : `连接状态：${state}`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
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
      toast.success("账号已删除");
      setDeleteTarget(null);
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  return (
    <PageScaffold subtitle="闲鱼多账号管理 — 列表 / 筛选 / 状态切换">
      <div className="space-y-4">
        {/* 工具栏 */}
        <div className="flex items-center justify-between gap-3">
          <div className="flex items-center gap-3">
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
          <div className="flex gap-2">
            <Button
              variant="outline"
              onClick={() => {
                setQrSeq((seq) => seq + 1);
                setQrOpen(true);
              }}
            >
              <QrCode className="size-4" aria-hidden />
              扫码登录
            </Button>
            <Button onClick={() => setShowForm(true)}>
              <Plus className="size-4" aria-hidden />
              新增账号
            </Button>
          </div>
        </div>

        {/* 列表 */}
        {loading ? (
          <Loading size="lg" text="加载中..." className="py-16" />
        ) : filtered.length === 0 ? (
          <div className="py-16 text-center text-muted-foreground">暂无账号</div>
        ) : (
          <div className="overflow-hidden rounded-xl border border-border">
            <table className="w-full text-[length:var(--text-sm)]">
              <thead className="bg-muted/50 text-muted-foreground">
                <tr>
                  <th className="px-4 py-2.5 text-left font-medium">账号 ID</th>
                  <th className="px-4 py-2.5 text-left font-medium">名称</th>
                  <th className="px-4 py-2.5 text-left font-medium">状态</th>
                  <th className="px-4 py-2.5 text-left font-medium">连接</th>
                  <th className="px-4 py-2.5 text-left font-medium">备注</th>
                  <th className="px-4 py-2.5 text-left font-medium">登录时间</th>
                  <th className="px-4 py-2.5 text-right font-medium">操作</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {filtered.map((account) => {
                  const conn = CONNECTION_LABELS[connectionStates[account.account_id] ?? "disconnected"];
                  const isConnecting = connectingId === account.account_id;
                  return (
                    <tr key={account.account_id} className="hover:bg-muted/30">
                      <td className="px-4 py-2.5 font-mono">{account.account_id}</td>
                      <td className="px-4 py-2.5">{account.display_name || "—"}</td>
                      <td className="px-4 py-2.5">
                        <span
                          className={
                            account.status === "active"
                              ? "rounded-full bg-emerald-500/15 px-2 py-0.5 text-[length:var(--text-xs)] text-emerald-500"
                              : "rounded-full bg-muted px-2 py-0.5 text-[length:var(--text-xs)] text-muted-foreground"
                          }
                        >
                          {account.status === "active" ? "启用" : "停用"}
                        </span>
                      </td>
                      <td className="px-4 py-2.5">
                        <span
                          className={`rounded-full px-2 py-0.5 text-[length:var(--text-xs)] ${conn.cls}`}
                          title={
                            connectionStates[account.account_id] === "error"
                              ? "连接异常，详情见运行日志（风控拦截会稍后自动重试）"
                              : undefined
                          }
                        >
                          {conn.label}
                        </span>
                      </td>
                      <td className="px-4 py-2.5 text-muted-foreground">{account.remark || "—"}</td>
                      <td className="px-4 py-2.5 text-right">
                        <div className="flex items-center justify-end gap-2">
                          {connectionStates[account.account_id] === "connected" ? (
                            <Button
                              size="sm"
                              variant="outline"
                              onClick={() => void handleDisconnect(account)}
                            >
                              断开
                            </Button>
                          ) : (
                            <Button
                              size="sm"
                              variant="outline"
                              disabled={isConnecting || !account.cookie}
                              onClick={() => void handleConnect(account)}
                            >
                              {isConnecting ? "连接中…" : "连接"}
                            </Button>
                          )}
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() => void handleToggleStatus(account)}
                          >
                            {account.status === "active" ? "停用" : "启用"}
                          </Button>
                          <Button
                            size="sm"
                            variant="ghost"
                            onClick={() => void handleUpdateRemark(account, account.remark)}
                          >
                            编辑
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
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* 新建账号弹窗 */}
      <ConfirmModal
        isOpen={showForm}
        title="新增闲鱼账号"
        message={
          <div className="space-y-3 text-left">
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">账号标识</span>
              <Input
                value={form.account_id}
                onChange={(event) => setForm({ ...form, account_id: event.target.value })}
                placeholder="如 acc-xianyu-001"
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">显示名称</span>
              <Input
                value={form.display_name}
                onChange={(event) => setForm({ ...form, display_name: event.target.value })}
                placeholder="我的闲鱼账号"
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">Cookie</span>
              <Input
                value={form.cookie}
                onChange={(event) => setForm({ ...form, cookie: event.target.value })}
                placeholder="unb=...; _m_h5_tk=..."
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">备注</span>
              <Input
                value={form.remark}
                onChange={(event) => setForm({ ...form, remark: event.target.value })}
                placeholder="可选"
              />
            </label>
          </div>
        }
        confirmText={saving ? "保存中…" : "创建"}
        loading={saving}
        onConfirm={() => void handleCreate()}
        onCancel={() => {
          setShowForm(false);
          setForm(EMPTY_FORM);
        }}
      />

      {/* 扫码登录弹窗 */}
      <AccountQrDialog
        key={qrSeq}
        open={qrOpen}
        onClose={() => setQrOpen(false)}
        onSuccess={() => void load()}
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

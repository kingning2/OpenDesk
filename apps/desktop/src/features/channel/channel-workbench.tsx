/**
 * 渠道平台工作区（二级页）— 平台内配置（浏览器快照登录 / 自动回复 / 连接）与会话工作区一体。
 *
 * 由 `/features/channel/:platform` 路由进入；通过 `kind` 读取平台注册表。
 * 本页聚合了该平台的全部操作：账号配置（快照导入）、自动回复开关、登录、连接、会话列表、消息流、人工发送。
 */

import { useEffect, useState } from "react";
import { Link, useParams } from "react-router";
import { ChevronRight, Plus, Trash2 } from "@desk/ui/icons";
import { Button, Input, PageScaffold } from "@desk/ui";
import { channelConnect, channelDisconnect, channelLogin, channelOpenSite, channelSend } from "@desk/platform/ipc/channel";
import type { ChannelAccount, ChannelMessage } from "@desk/contracts";
import { useChannelEvents } from "./use-channel-events";
import { useChannelStore } from "./use-channel-store";
import { QrLoginDialog } from "./qr-login-dialog";
import { getChannelPlatform } from "./platforms";
import { parseSnapshot } from "./snapshot";

/** 凭据展示摘要：快照 JSON 显示 cookies 数量，旧字符串打码。 */
function maskCredential(credential: string): string {
  if (!credential) return "—";
  const parsed = parseSnapshot(credential);
  if (parsed.ok && parsed.cookies.length > 0) {
    return `已配置 ${parsed.cookies.length} 个 cookie`;
  }
  // 旧 cookie 字符串打码。
  if (credential.length <= 16) return "••••••••";
  return `${credential.slice(0, 8)}••••••${credential.slice(-4)}`;
}

function formatTime(timestamp: string): string {
  const millis = Number(timestamp);
  if (!Number.isNaN(millis) && millis > 1_000_000_000_000) {
    return new Date(millis).toLocaleTimeString("zh-CN", {
      hour: "2-digit",
      minute: "2-digit",
    });
  }
  return timestamp;
}

/** 账号表单（Cookie 配置）。 */
function AccountForm({
  account,
  kind,
  onDone,
}: {
  account: ChannelAccount | null;
  kind: string;
  onDone: () => void;
}) {
  const upsertAccount = useChannelStore((state) => state.upsertAccount);

  const [name, setName] = useState(account?.name ?? "");
  const [credential, setCredential] = useState(account?.credential ?? "");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [snapshotHint, setSnapshotHint] = useState<string | null>(null);

  // 输入时实时校验快照。
  function handleCredentialChange(value: string) {
    setCredential(value);
    const result = parseSnapshot(value);
    if (value.trim() && !result.ok) {
      setSnapshotHint(result.detail);
    } else if (value.trim() && result.ok) {
      setSnapshotHint(result.detail);
    } else {
      setSnapshotHint(null);
    }
  }

  async function save() {
    if (!name.trim() || !credential.trim()) {
      setError("名称与凭据不能为空");
      return;
    }
    const parsed = parseSnapshot(credential);
    if (!parsed.ok) {
      setError(parsed.detail);
      return;
    }
    setSaving(true);
    setError(null);
    try {
      await upsertAccount({
        id: account?.id ?? crypto.randomUUID(),
        kind,
        name: name.trim(),
        credential: credential.trim(),
        enabled: account?.enabled ?? true,
      });
      onDone();
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : String(saveError));
      setSaving(false);
    }
  }

  return (
    <div className="space-y-3">
      <label className="block space-y-1">
        <span className="text-[length:var(--text-sm)] text-muted-foreground">账号名称</span>
        <Input value={name} onChange={(event) => setName(event.target.value)} placeholder="我的闲鱼账号" />
      </label>
      <label className="block space-y-1">
        <span className="text-[length:var(--text-sm)] text-muted-foreground">
          登录快照（Chrome 扩展导出 JSON）
        </span>
        <textarea
          value={credential}
          onChange={(event) => handleCredentialChange(event.target.value)}
          placeholder='粘贴闲鱼登录快照 JSON（含 cookies / env / headers）。旧 Cookie 字符串仍兼容。'
          rows={6}
          className="w-full resize-y rounded-[var(--radius-md)] border border-border bg-background px-3 py-2 font-mono text-[length:var(--text-xs)] outline-none focus:ring-2 focus:ring-ring"
        />
      </label>
      {snapshotHint ? (
        <p className={`text-[length:var(--text-xs)] ${credential.trim() && parseSnapshot(credential).ok ? "text-emerald-600" : "text-muted-foreground"}`}>
          {snapshotHint}
        </p>
      ) : null}
      {error ? <p className="text-[length:var(--text-sm)] text-destructive">{error}</p> : null}
      <div className="flex justify-end gap-2">
        <Button variant="ghost" onClick={onDone}>取消</Button>
        <Button onClick={save} disabled={saving}>{saving ? "保存中…" : "保存"}</Button>
      </div>
    </div>
  );
}

/** 消息气泡。 */
function MessageBubble({ message }: { message: ChannelMessage }) {
  const isInbound = message.direction === "in";
  return (
    <div className={`flex ${isInbound ? "justify-start" : "justify-end"}`}>
      <div
        className={`max-w-[75%] rounded-[var(--radius-lg)] px-3 py-2 text-[length:var(--text-sm)] ${
          isInbound
            ? "border border-border/70 bg-card text-foreground"
            : "bg-primary text-primary-foreground"
        }`}
      >
        <p className="whitespace-pre-wrap break-words">{message.content}</p>
        <p className="mt-0.5 text-right text-[length:var(--text-xs)] opacity-60">
          {isInbound ? "买家" : message.sender === "ai" ? "AI" : "我"} · {formatTime(message.created_at)}
        </p>
      </div>
    </div>
  );
}

/**
 * 平台工作区页面。
 */
export function ChannelWorkbench() {
  useChannelEvents();

  const { platform: platformPath } = useParams<{ platform: string }>();
  const platform = getChannelPlatform(platformPath ?? "");

  const accounts = useChannelStore((state) => state.accounts);
  const conversations = useChannelStore((state) => state.conversations);
  const messages = useChannelStore((state) => state.messages);
  const settings = useChannelStore((state) => state.settings);
  const load = useChannelStore((state) => state.load);
  const loaded = useChannelStore((state) => state.loaded);
  const error = useChannelStore((state) => state.error);
  const activeConversationId = useChannelStore((state) => state.activeConversationId);
  const selectConversation = useChannelStore((state) => state.selectConversation);
  const setAutoReply = useChannelStore((state) => state.setAutoReply);
  const removeAccount = useChannelStore((state) => state.removeAccount);

  const [status, setStatus] = useState("disconnected");
  const [connecting, setConnecting] = useState(false);
  const [draft, setDraft] = useState("");
  const [suggestion, setSuggestion] = useState<string | null>(null);
  const [sending, setSending] = useState(false);
  const [editing, setEditing] = useState<ChannelAccount | "new" | null>(null);
  const [loginError, setLoginError] = useState<string | null>(null);
  const [qrOpen, setQrOpen] = useState(false);
  const [qrSessionKey, setQrSessionKey] = useState(0);

  useEffect(() => {
    if (!loaded) {
      void load();
    }
  }, [loaded, load]);

  useEffect(() => {
    function onSuggestion(event: Event) {
      const detail = (event as CustomEvent<string>).detail;
      setSuggestion(detail);
    }
    window.addEventListener("channel:suggestion", onSuggestion);
    return () => window.removeEventListener("channel:suggestion", onSuggestion);
  }, []);

  // 未知平台路径 → 引导回选择页。
  if (!platform) {
    return (
      <PageScaffold subtitle="未知平台">
        <p className="text-[length:var(--text-sm)] text-muted-foreground">
          未找到该平台。请返回渠道首页选择。
        </p>
        <Link
          to="/features/channel"
          className="mt-3 inline-block rounded-[var(--radius-md)] bg-primary px-4 py-2 text-[length:var(--text-sm)] text-primary-foreground"
        >
          返回平台选择
        </Link>
      </PageScaffold>
    );
  }

  // 平台账号（按 kind 过滤）。
  const platformAccounts = accounts.filter((account) => account.kind === platform.kind);
  const primaryAccount = platformAccounts.find((account) => account.enabled) ?? platformAccounts[0];
  const activeConversation = conversations.find(
    (conversation) => conversation.id === activeConversationId,
  );
  const activeMessages = activeConversationId
    ? messages.filter((message) => message.conversation_id === activeConversationId)
    : [];

  async function handleConnect() {
    if (!primaryAccount || connecting) {
      return;
    }
    setConnecting(true);
    setStatus("connecting");
    try {
      const result = await channelConnect(primaryAccount.id);
      setStatus(result.ok ? result.state : "error");
    } catch (connectError) {
      setStatus("error");
      console.error(connectError);
    } finally {
      setConnecting(false);
    }
  }

  async function handleDisconnect() {
    if (!primaryAccount) {
      return;
    }
    try {
      await channelDisconnect(primaryAccount.id);
      setStatus("disconnected");
    } catch (disconnectError) {
      console.error(disconnectError);
    }
  }

  async function handleLogin() {
    if (!primaryAccount || connecting) {
      return;
    }
    setConnecting(true);
    setStatus("connecting");
    setLoginError(null);
    try {
      const result = await channelLogin({ account_id: primaryAccount.id });
      setStatus(result.ok ? result.state : "error");
      if (!result.ok) {
        setLoginError(result.detail ?? "登录失败");
      }
      await load();
    } catch (loginError) {
      setStatus("error");
      setLoginError(loginError instanceof Error ? loginError.message : String(loginError));
    } finally {
      setConnecting(false);
    }
  }

  async function handleOpenSite() {
    if (!primaryAccount) {
      return;
    }
    try {
      await channelOpenSite({ account_id: primaryAccount.id });
    } catch (openError) {
      setLoginError(openError instanceof Error ? openError.message : String(openError));
    }
  }

  async function handleSend() {
    const text = draft.trim();
    if (!text || !activeConversation || sending) {
      return;
    }
    setSending(true);
    setSuggestion(null);
    try {
      await channelSend({ conversation_id: activeConversation.id, content: text });
      setDraft("");
    } catch (sendError) {
      console.error(sendError);
    } finally {
      setSending(false);
    }
  }

  const Icon = platform.icon;

  return (
    <PageScaffold subtitle={`${platform.name} 客服工作区`}>
      {/* 面包屑 + 平台标题 */}
      <div className="mb-4 flex items-center gap-1.5 text-[length:var(--text-sm)] text-muted-foreground">
        <Link to="/features/channel" className="hover:text-foreground">渠道</Link>
        <ChevronRight className="size-3.5" aria-hidden />
        <span className="font-medium text-foreground">{platform.name}</span>
      </div>

      <div className="grid min-h-0 flex-1 grid-cols-[300px_1fr] gap-4">
        {/* 左栏：平台配置 + 会话列表 */}
        <aside className="flex min-h-0 flex-col gap-4">
          {/* 账号与连接配置 */}
          <section className="rounded-[var(--radius-xl)] border border-border/70 bg-card p-4">
            <div className="mb-3 flex items-center justify-between">
              <h3 className="flex items-center gap-2 text-[length:var(--text-base)] font-semibold tracking-tight">
                <Icon className="size-4" aria-hidden /> 账号
              </h3>
              <Button variant="ghost" size="sm" onClick={() => setEditing("new")}>
                <Plus className="size-3.5" aria-hidden /> 添加
              </Button>
            </div>

            {platformAccounts.length === 0 ? (
              <p className="text-[length:var(--text-xs)] text-muted-foreground">
                尚未配置 {platform.name} 账号。
              </p>
            ) : (
              <div className="space-y-2">
                {platformAccounts.map((account) => (
                  <div
                    key={account.id}
                    className="rounded-[var(--radius-md)] border border-border/70 px-3 py-2"
                  >
                    <div className="flex items-center justify-between">
                      <p className="truncate text-[length:var(--text-sm)] font-medium">{account.name}</p>
                      <div className="flex shrink-0 items-center gap-1">
                        <Button variant="ghost" size="sm" onClick={() => setEditing(account)}>编辑</Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          className="text-destructive"
                          onClick={async () => {
                            if (primaryAccount?.id === account.id && status === "connected") {
                              await handleDisconnect();
                            }
                            await removeAccount(account.id);
                          }}
                        >
                          <Trash2 className="size-3.5" aria-hidden />
                        </Button>
                      </div>
                    </div>
                    <p className="truncate font-mono text-[length:var(--text-xs)] text-muted-foreground">
                      {maskCredential(account.credential)}
                    </p>
                  </div>
                ))}
              </div>
            )}

            {editing ? (
              <div className="mt-3 rounded-[var(--radius-lg)] border border-border/70 p-3">
                <AccountForm
                  account={editing === "new" ? null : editing}
                  kind={platform.kind}
                  onDone={() => {
                    setEditing(null);
                    void load();
                  }}
                />
              </div>
            ) : null}

            {/* 连接控制 + 自动回复 */}
            <div className="mt-4 space-y-3 border-t border-border/70 pt-3">
              {loginError ? (
                <p className="text-[length:var(--text-xs)] text-destructive">{loginError}</p>
              ) : null}

              <div className="flex items-center justify-between">
                <span className="flex items-center gap-2 text-[length:var(--text-sm)]">
                  <span
                    className={`size-2 rounded-full ${
                      status === "connected"
                        ? "bg-emerald-500"
                        : status === "connecting"
                          ? "bg-amber-500"
                          : "bg-muted-foreground/50"
                    }`}
                    aria-hidden
                  />
                  {status === "connected" ? "已连接" : status === "connecting" ? "连接中…" : "未连接"}
                </span>
                <div className="flex gap-2">
                  <Button size="sm" disabled={!primaryAccount} onClick={() => {
                    setQrSessionKey((key) => key + 1);
                    setQrOpen(true);
                  }}>
                    扫码登录
                  </Button>
                  <Button size="sm" variant="outline" disabled={connecting || !primaryAccount} onClick={handleLogin}>
                    {connecting ? "登录中…" : "快照登录"}
                  </Button>
                  <Button
                    size="sm"
                    disabled={connecting || !primaryAccount}
                    onClick={handleConnect}
                  >
                    {status === "connected" ? "已连接" : connecting ? "连接中…" : "连接"}
                  </Button>
                  {status === "connected" ? (
                    <Button variant="outline" size="sm" onClick={handleDisconnect}>断开</Button>
                  ) : null}
                </div>
              </div>

              <Button size="sm" variant="ghost" disabled={!primaryAccount} onClick={handleOpenSite}>
                打开闲鱼页面
              </Button>

              <div className="flex items-center justify-between">
                <div>
                  <p className="text-[length:var(--text-sm)] font-medium">自动回复</p>
                  <p className="text-[length:var(--text-xs)] text-muted-foreground">
                    入站消息由 AI 自动回复；关闭后仅生成建议。
                  </p>
                </div>
                <label className="relative inline-flex cursor-pointer items-center">
                  <input
                    type="checkbox"
                    className="peer sr-only"
                    checked={settings.auto_reply}
                    onChange={(event) => setAutoReply(event.target.checked)}
                  />
                  <span className="h-5 w-9 rounded-full bg-border transition-colors peer-checked:bg-primary peer-focus-visible:ring-2 peer-focus-visible:ring-ring" />
                </label>
              </div>
            </div>
          </section>

          {/* 会话列表 */}
          <section className="min-h-0 flex-1 overflow-y-auto rounded-[var(--radius-xl)] border border-border/70 bg-card">
            {conversations.length === 0 ? (
              <p className="p-4 text-[length:var(--text-sm)] text-muted-foreground">
                暂无会话。连接后入站消息会出现在这里。
              </p>
            ) : (
              conversations.map((conversation) => (
                <button
                  type="button"
                  key={conversation.id}
                  onClick={() => selectConversation(conversation.id)}
                  className={`block w-full border-b border-border/50 px-3 py-2.5 text-left transition-colors hover:bg-muted/40 ${
                    activeConversationId === conversation.id ? "bg-muted/60" : ""
                  }`}
                >
                  <p className="truncate text-[length:var(--text-sm)] font-medium">
                    {conversation.peer_name || conversation.peer_id}
                  </p>
                  <p className="truncate text-[length:var(--text-xs)] text-muted-foreground">
                    {conversation.item_title || `商品 ${conversation.item_id ?? "?"}`}
                  </p>
                </button>
              ))
            )}
          </section>
        </aside>

        {/* 右栏：消息流 + 输入 */}
        <section className="flex min-h-0 flex-col overflow-hidden rounded-[var(--radius-xl)] border border-border/70 bg-card">
          {error ? (
            <p className="border-b border-border/70 px-4 py-2 text-[length:var(--text-sm)] text-destructive">
              加载失败：{error}
            </p>
          ) : null}

          <div className="min-h-0 flex-1 space-y-3 overflow-y-auto p-4">
            {!activeConversation ? (
              <p className="py-10 text-center text-[length:var(--text-sm)] text-muted-foreground">
                选择一个会话查看消息。
              </p>
            ) : activeMessages.length === 0 ? (
              <p className="py-10 text-center text-[length:var(--text-sm)] text-muted-foreground">
                暂无消息。
              </p>
            ) : (
              activeMessages.map((message) => (
                <MessageBubble key={message.id} message={message} />
              ))
            )}
          </div>

          {suggestion ? (
            <div className="flex items-center justify-between border-t border-border/70 px-4 py-2">
              <p className="min-w-0 flex-1 truncate text-[length:var(--text-sm)] text-muted-foreground">
                AI 建议：{suggestion}
              </p>
              <button
                type="button"
                onClick={() => {
                  if (activeConversation) {
                    setDraft(suggestion);
                  }
                  setSuggestion(null);
                }}
                className="ml-3 shrink-0 rounded-[var(--radius-md)] border border-border px-2.5 py-1 text-[length:var(--text-xs)]"
              >
                使用
              </button>
            </div>
          ) : null}

          <div className="flex items-center gap-2 border-t border-border/70 p-3">
            <textarea
              value={draft}
              onChange={(event) => setDraft(event.target.value)}
              placeholder={activeConversation ? "输入回复…" : "先选择会话"}
              disabled={!activeConversation}
              rows={2}
              className="min-h-0 flex-1 resize-none rounded-[var(--radius-md)] border border-border bg-background px-3 py-2 text-[length:var(--text-sm)] outline-none focus:ring-2 focus:ring-ring"
            />
            <button
              type="button"
              disabled={!draft.trim() || !activeConversation || sending}
              onClick={handleSend}
              className="shrink-0 rounded-[var(--radius-md)] bg-primary px-4 py-2 text-[length:var(--text-sm)] text-primary-foreground disabled:opacity-50"
            >
              {sending ? "发送中…" : "发送"}
            </button>
          </div>
        </section>
      </div>

      <QrLoginDialog
        key={qrSessionKey}
        open={qrOpen}
        accountId={primaryAccount?.id ?? ""}
        onClose={() => setQrOpen(false)}
        onSuccess={() => {
          setStatus("connected");
          void load();
        }}
      />
    </PageScaffold>
  );
}

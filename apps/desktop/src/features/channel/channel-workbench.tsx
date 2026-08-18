/**
 * 渠道平台工作区（二级页）— 平台内配置（扫码登录 / 自动回复 / 连接）与会话工作区一体。
 *
 * 由编译期静态路径进入（`OPENDESK_CHANNEL_PLATFORM`）；通过 `kind` 读取平台注册表。
 * 本页聚合了该平台的全部操作：账号配置（扫码登录）、自动回复开关、连接、会话列表、消息流、人工发送；
 * 并可在主窗口右侧内嵌闲鱼站点（子 WebView + cookie）。
 *
 * @author Xiaoman
 * @created 2026-08-13
 */

import { useEffect, useRef, useState } from "react";
import { Link } from "react-router";
import { CHANNEL_WORKBENCH_PATH, getActiveChannelPlatform, managePath } from "@desk/platform/compile";
import { ChevronRight, Plus, Trash2 } from "@desk/ui/icons";
import { Button, Input, ScrollArea, Switch, Textarea } from "@desk/ui";
import {
  channelCloseSite,
  channelConnect,
  channelDisconnect,
  channelOpenSite,
  channelSend,
} from "@desk/platform/ipc/channel";
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

/** 账号表单（仅配置名称；凭据由扫码登录写入）。 */
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
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function save() {
    if (!name.trim()) {
      setError("账号名称不能为空");
      return;
    }
    setSaving(true);
    setError(null);
    try {
      await upsertAccount({
        id: account?.id ?? crypto.randomUUID(),
        kind,
        name: name.trim(),
        credential: account?.credential ?? "",
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

  const platform = getChannelPlatform(getActiveChannelPlatform());

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
  /** 是否在主窗口右侧内嵌闲鱼站点。 */
  const [siteOpen, setSiteOpen] = useState(false);
  const siteHostRef = useRef<HTMLDivElement>(null);

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

  const siteAccountId =
    platform == null
      ? null
      : (accounts.find((item) => item.kind === platform.kind && item.enabled)?.id ??
        accounts.find((item) => item.kind === platform.kind)?.id ??
        null);

  // 关闭：仅当 siteOpen=false，或页面真正卸载时销毁（bounds 同步 effect 绝不 close）。
  useEffect(() => {
    if (!siteOpen) {
      void channelCloseSite().catch(() => {});
    }
  }, [siteOpen]);

  useEffect(() => {
    return () => {
      void channelCloseSite().catch(() => {});
    };
  }, []);

  // 内嵌闲鱼：测量占位 DOM → 同步子 WebView bounds（幂等打开）。
  useEffect(() => {
    if (!siteOpen || !siteAccountId) {
      return;
    }
    const accountId = siteAccountId;
    let cancelled = false;
    let syncTimer: ReturnType<typeof setTimeout> | null = null;
    let observer: ResizeObserver | null = null;

    async function syncBounds() {
      const host = siteHostRef.current;
      if (!host || cancelled) {
        return;
      }
      const rect = host.getBoundingClientRect();
      if (rect.width < 8 || rect.height < 8) {
        return;
      }
      try {
        await channelOpenSite({
          account_id: accountId,
          x: rect.x,
          y: rect.y,
          width: rect.width,
          height: rect.height,
        });
        if (!cancelled) {
          setLoginError(null);
        }
      } catch (openError) {
        if (!cancelled) {
          setLoginError(openError instanceof Error ? openError.message : String(openError));
          setSiteOpen(false);
        }
      }
    }

    function scheduleSync() {
      if (syncTimer) {
        clearTimeout(syncTimer);
      }
      syncTimer = setTimeout(() => {
        void syncBounds();
      }, 80);
    }

    const frame = requestAnimationFrame(() => {
      void syncBounds();
      const host = siteHostRef.current;
      if (host && !observer) {
        observer = new ResizeObserver(scheduleSync);
        observer.observe(host);
      }
    });
    window.addEventListener("resize", scheduleSync);

    return () => {
      cancelled = true;
      cancelAnimationFrame(frame);
      if (syncTimer) {
        clearTimeout(syncTimer);
      }
      observer?.disconnect();
      window.removeEventListener("resize", scheduleSync);
    };
  }, [siteOpen, siteAccountId]);

  // 未知平台路径 → 引导回选择页。
  if (!platform) {
    return (
      <>
      <p className="text-(length:--text-sm) text-muted-foreground">未知平台</p>
      <p className="text-[length:var(--text-sm)] text-muted-foreground">
          未找到该平台。请返回渠道首页选择。
        </p>
        <Link
          to={CHANNEL_WORKBENCH_PATH}
          className="mt-3 inline-block rounded-[var(--radius-md)] bg-primary px-4 py-2 text-[length:var(--text-sm)] text-primary-foreground"
        >
          返回会话工作台
        </Link>
    </>
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

  function handleToggleSite() {
    if (!primaryAccount) {
      return;
    }
    if (!primaryAccount.credential.trim()) {
      setLoginError("请先扫码登录以获取 cookies");
      return;
    }
    setLoginError(null);
    setSiteOpen((open) => !open);
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
    <>
      <p className="text-(length:--text-sm) text-muted-foreground">{`${platform.name} 会话工作台`}</p>
      {/* 平台标题 + 管理后台入口 */}
      <div className="mb-4 flex items-center gap-1.5 text-[length:var(--text-sm)] text-muted-foreground">
        <span className="font-medium text-foreground">{platform.name}</span>
        {platform.kind === "xianyu" ? (
          <>
            <ChevronRight className="size-3.5" aria-hidden />
            <Link
              to={managePath("dashboard")}
              className="hover:text-foreground"
            >
              业务管理
            </Link>
          </>
        ) : null}
      </div>

      <div
        className={`grid min-h-0 flex-1 gap-4 ${
          siteOpen ? "grid-cols-[280px_minmax(0,1fr)_minmax(360px,1.2fr)]" : "grid-cols-[300px_1fr]"
        }`}
      >
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
                  <Button size="sm" onClick={() => {
                    setQrSessionKey((key) => key + 1);
                    setQrOpen(true);
                  }}>
                    扫码登录
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

              <Button size="sm" variant="ghost" disabled={!primaryAccount} onClick={handleToggleSite}>
                {siteOpen ? "关闭闲鱼页面" : "打开闲鱼页面"}
              </Button>

              <div className="flex items-center justify-between">
                <div>
                  <p className="text-[length:var(--text-sm)] font-medium">自动回复</p>
                  <p className="text-[length:var(--text-xs)] text-muted-foreground">
                    入站消息由 AI 自动回复；关闭后仅生成建议。
                  </p>
                </div>
                <Switch
                  checked={settings.auto_reply}
                  onCheckedChange={setAutoReply}
                  aria-label="自动回复"
                />
              </div>
            </div>
          </section>

          {/* 会话列表 */}
          <section className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-[var(--radius-xl)] border border-border/70 bg-card">
            <ScrollArea className="min-h-0 flex-1">
            {conversations.length === 0 ? (
              <p className="p-4 text-[length:var(--text-sm)] text-muted-foreground">
                暂无会话。连接后入站消息会出现在这里。
              </p>
            ) : (
              conversations.map((conversation) => (
                <Button
                  variant="ghost"
                  key={conversation.id}
                  onClick={() => selectConversation(conversation.id)}
                  className={`h-auto w-full justify-start rounded-none border-b border-border/50 px-3 py-2.5 text-left font-normal ${
                    activeConversationId === conversation.id ? "bg-muted/60" : ""
                  }`}
                >
                  <span className="block min-w-0">
                    <span className="block truncate text-[length:var(--text-sm)] font-medium">
                      {conversation.peer_name || conversation.peer_id}
                    </span>
                    <span className="block truncate text-[length:var(--text-xs)] text-muted-foreground">
                      {conversation.item_title || `商品 ${conversation.item_id ?? "?"}`}
                    </span>
                  </span>
                </Button>
              ))
            )}
            </ScrollArea>
          </section>
        </aside>

        {/* 右栏：消息流 + 输入 */}
        <section className="flex min-h-0 flex-col overflow-hidden rounded-[var(--radius-xl)] border border-border/70 bg-card">
          {error ? (
            <p className="border-b border-border/70 px-4 py-2 text-[length:var(--text-sm)] text-destructive">
              加载失败：{error}
            </p>
          ) : null}

          <ScrollArea className="min-h-0 flex-1">
            <div className="space-y-3 p-4">
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
          </ScrollArea>

          {suggestion ? (
            <div className="flex items-center justify-between border-t border-border/70 px-4 py-2">
              <p className="min-w-0 flex-1 truncate text-[length:var(--text-sm)] text-muted-foreground">
                AI 建议：{suggestion}
              </p>
              <Button
                size="sm"
                variant="outline"
                onClick={() => {
                  if (activeConversation) {
                    setDraft(suggestion);
                  }
                  setSuggestion(null);
                }}
                className="ml-3 shrink-0"
              >
                使用
              </Button>
            </div>
          ) : null}

          <div className="flex items-center gap-2 border-t border-border/70 p-3">
            <Textarea
              value={draft}
              onChange={(event) => setDraft(event.target.value)}
              placeholder={activeConversation ? "输入回复…" : "先选择会话"}
              disabled={!activeConversation}
              rows={2}
              className="min-h-0 flex-1 resize-none"
            />
            <Button
              disabled={!draft.trim() || !activeConversation || sending}
              onClick={handleSend}
              className="shrink-0"
            >
              {sending ? "发送中…" : "发送"}
            </Button>
          </div>
        </section>

        {siteOpen ? (
          <section className="flex min-h-0 flex-col overflow-hidden rounded-[var(--radius-xl)] border border-border/70 bg-card">
            <div className="flex items-center justify-between border-b border-border/70 px-3 py-2">
              <p className="text-[length:var(--text-sm)] font-medium">闲鱼页面</p>
              <Button size="sm" variant="ghost" onClick={() => setSiteOpen(false)}>
                关闭
              </Button>
            </div>
            {/* 原生子 WebView 叠在此占位上；勿在内部放可交互内容以免错位。 */}
            <div
              ref={siteHostRef}
              className="min-h-0 flex-1 bg-muted/30"
              aria-label="闲鱼内嵌视图占位"
            />
          </section>
        ) : null}
      </div>

      <QrLoginDialog
        key={qrSessionKey}
        open={qrOpen}
        accountId={primaryAccount?.id ?? ""}
        name={primaryAccount?.name ?? `${platform.name}账号`}
        kind={platform.kind}
        onClose={() => setQrOpen(false)}
        onSuccess={() => {
          setStatus("connected");
          void load();
        }}
      />
    </>
  );
}

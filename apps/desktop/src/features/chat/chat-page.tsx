/**
 * 客户会话收件箱 — 会话列表、消息记录、人工发送与客户信息侧栏。
 *
 * 复用 @desk/ui 控件与 Aceternity 风格光晕卡片（PageGlowCard），结构对齐账号管理等
 * 管理页（标准 PageScaffold 内边距 + 令牌字号 + 卡片容器）；消息气泡等 Aceternity
 * 未覆盖的部分手写。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */

import { useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import {
  AsyncButton,
  Button,
  IconButton,
  Loading,
  PageGlowCard,
  PageScaffold,
  ScrollArea,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Textarea,
  cn,
  toast,
} from "@desk/ui";
import {
  ChevronDown,
  ChevronRight,
  MessageSquare,
  Package,
  PanelRightClose,
  PanelRightOpen,
  RefreshCw,
  Send,
  ShoppingCart,
  User,
  type LucideIcon,
} from "@desk/ui/icons";
import type { ChannelConversation } from "@desk/contracts";
import { managePath } from "@desk/platform/compile";
import { accountList, type XianyuAccount } from "@desk/platform/ipc/account";
import {
  channelProductHeadinfo,
  type ProductHeadInfo,
} from "@desk/platform/ipc/channel";
import { orderList, type Order } from "@desk/platform/ipc/order";
import { formatAmount } from "@desk/utils";
import { useWorkspaceNav } from "../../app/use-workspace-tabs";
import {
  conversationNeedsReply,
  lastMessagePreview,
  useChannelInbox,
} from "./use-channel-inbox";

const OWNER_ID = 1;

const ORDER_STATUS_LABELS: Record<string, string> = {
  pending: "待付款",
  paid: "待发货",
  shipped: "已发货",
  completed: "已完成",
  closed: "已关闭",
  refunded: "已退款",
  unknown: "未知",
};

/**
 * 格式化消息时间（created_at 为毫秒字符串）。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
function formatMessageTime(createdAt: string): string {
  const ms = Number(createdAt);
  if (!Number.isFinite(ms) || ms <= 0) {
    return "";
  }
  return new Date(ms).toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/**
 * 格式化 ISO 时间（订单下单时间等）。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
function formatDateTime(value?: string | null): string {
  if (!value) {
    return "";
  }
  const time = new Date(value).getTime();
  if (!Number.isFinite(time)) {
    return "";
  }
  return new Date(time).toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/**
 * 圆形头像：有 URL 显示图片，否则显示名称首字。
 *
 * @author Xiaoman
 * @created 2026-08-21
 */
function ConversationAvatar({
  name,
  src,
  size = "md",
}: {
  name: string;
  src?: string | null;
  size?: "sm" | "md";
}) {
  const dim = size === "sm" ? "size-7" : "size-9";
  const text = size === "sm" ? "text-[length:var(--text-xs)]" : "text-[length:var(--text-sm)]";
  const initial = (name.trim() || "?").slice(0, 1);
  if (src) {
    return (
      <img
        src={src}
        alt=""
        className={cn(dim, "shrink-0 rounded-full border border-border object-cover")}
      />
    );
  }
  return (
    <div
      className={cn(
        dim,
        text,
        "flex shrink-0 items-center justify-center rounded-full border border-border bg-muted font-medium text-muted-foreground",
      )}
      aria-hidden
    >
      {initial}
    </div>
  );
}

/**
 * 客户信息区块 — Aceternity 光晕卡片外壳（内层结构对齐账号管理 PageGlowCard）。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
function InfoSection({
  icon: Icon,
  title,
  children,
}: {
  icon: LucideIcon;
  title: string;
  children: ReactNode;
}) {
  return (
    <PageGlowCard className="h-full">
      <div className="relative h-full rounded-[inherit] border border-border bg-card p-4">
        <div className="flex items-center gap-2 text-[length:var(--text-sm)] font-medium">
          <Icon className="size-4 text-primary" aria-hidden />
          <span>{title}</span>
        </div>
        <div className="mt-3 space-y-2">{children}</div>
      </div>
    </PageGlowCard>
  );
}

/**
 * 信息行 — 左标签右值。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
function InfoRow({
  label,
  value,
  mono,
}: {
  label: string;
  value?: string | null;
  mono?: boolean;
}) {
  return (
    <div className="flex items-start justify-between gap-3 text-[length:var(--text-xs)]">
      <span className="shrink-0 text-muted-foreground">{label}</span>
      <span className={cn("min-w-0 break-all text-right", mono && "font-mono")}>
        {value || "—"}
      </span>
    </div>
  );
}

/**
 * 客户会话页。
 *
 * @author Xiaoman
 * @created 2026-08-20
 *
 * @returns 页面节点
 */
export function ChatPage() {
  const { selectTab } = useWorkspaceNav();
  const {
    loading,
    error,
    messages,
    selectedId,
    selectedConversation,
    threadMessages,
    refresh,
    selectConversation,
    sendMessage,
    accountFilter,
    setAccountFilter,
    filteredConversations,
  } = useChannelInbox();

  const [accounts, setAccounts] = useState<XianyuAccount[]>([]);
  const [draft, setDraft] = useState("");
  const [sending, setSending] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [infoOpen, setInfoOpen] = useState(false);
  const [buyerOrders, setBuyerOrders] = useState<Order[]>([]);
  const [ordersLoading, setOrdersLoading] = useState(false);
  const [ordersError, setOrdersError] = useState<string | null>(null);
  const [headInfo, setHeadInfo] = useState<ProductHeadInfo | null>(null);
  /** 折叠的账号 id 集合（未列入则展开）。 */
  const [collapsedAccountIds, setCollapsedAccountIds] = useState<Set<string>>(
    () => new Set(),
  );
  const threadEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    void accountList(OWNER_ID)
      .then(setAccounts)
      .catch(() => setAccounts([]));
  }, []);

  useEffect(() => {
    threadEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [threadMessages, selectedId]);

  const accountMetaById = useMemo(() => {
    const map = new Map<string, { label: string; avatarUrl: string }>();
    for (const account of accounts) {
      map.set(account.account_id, {
        label: account.display_name || account.account_id,
        avatarUrl: account.avatar_url || "",
      });
    }
    return map;
  }, [accounts]);

  // 会话按账号分组，侧栏一眼看出归属账号。
  const groupedConversations = useMemo(() => {
    const groups = new Map<string, ChannelConversation[]>();
    for (const conversation of filteredConversations) {
      const list = groups.get(conversation.account_id) ?? [];
      list.push(conversation);
      groups.set(conversation.account_id, list);
    }
    return [...groups.entries()].map(([accountId, list]) => {
      const meta = accountMetaById.get(accountId);
      return {
        accountId,
        label: meta?.label ?? accountId,
        avatarUrl: meta?.avatarUrl ?? "",
        list,
      };
    });
  }, [filteredConversations, accountMetaById]);

  function toggleAccountGroup(accountId: string) {
    setCollapsedAccountIds((current) => {
      const next = new Set(current);
      if (next.has(accountId)) {
        next.delete(accountId);
      } else {
        next.add(accountId);
      }
      return next;
    });
  }

  const selectedPeerId = selectedConversation?.peer_id ?? "";

  // 展开客户信息栏时，按买家 ID 拉取该买家订单。
  useEffect(() => {
    if (!infoOpen || !selectedPeerId) {
      setBuyerOrders([]);
      return;
    }
    let cancelled = false;
    setOrdersLoading(true);
    setOrdersError(null);
    orderList({ owner_id: OWNER_ID, page: 1, page_size: 50, buyer_id: selectedPeerId })
      .then(([list]) => {
        if (!cancelled) {
          setBuyerOrders(list);
        }
      })
      .catch((cause) => {
        if (!cancelled) {
          setOrdersError(cause instanceof Error ? cause.message : String(cause));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setOrdersLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [infoOpen, selectedPeerId]);

  // 展开客户信息栏时，拉取会话关联商品卡信息（message.headinfo）。
  useEffect(() => {
    if (!infoOpen || !selectedId) {
      setHeadInfo(null);
      return;
    }
    let cancelled = false;
    channelProductHeadinfo(selectedId)
      .then((data) => {
        if (!cancelled) {
          setHeadInfo(data);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setHeadInfo(null);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [infoOpen, selectedId]);

  const firstOrder = buyerOrders[0];
  const buyerFishNick = firstOrder?.buyer_fish_nick?.trim() || undefined;
  const buyerNick = firstOrder?.buyer_nick?.trim() || undefined;
  const peerName =
    selectedConversation?.peer_name?.trim() ||
    selectedConversation?.peer_id ||
    "未知联系人";
  const accountLabel = selectedConversation
    ? (accountMetaById.get(selectedConversation.account_id)?.label ??
      selectedConversation.account_id)
    : "";
  const needsReply = selectedConversation
    ? conversationNeedsReply(selectedConversation.id, messages)
    : false;
  const lastMessageTime = threadMessages.length
    ? formatMessageTime(threadMessages[threadMessages.length - 1].created_at)
    : "";
  const itemPrice =
    selectedConversation?.item_price != null
      ? formatAmount(selectedConversation.item_price)
      : undefined;

  // headinfo 商品卡字段（响应结构以实际返回为准，字段做了常见名兜底）。
  const headTitle =
    (typeof headInfo?.title === "string" && headInfo.title) ||
    (typeof headInfo?.itemTitle === "string" && headInfo.itemTitle) ||
    undefined;
  const headPrice =
    headInfo?.price != null && String(headInfo.price) ? String(headInfo.price) : undefined;
  const headImage =
    (typeof headInfo?.image === "string" && headInfo.image) ||
    (typeof headInfo?.mainImg === "string" && headInfo.mainImg) ||
    (typeof headInfo?.pic === "string" && headInfo.pic) ||
    undefined;

  async function handleRefresh() {
    setRefreshing(true);
    try {
      await refresh();
    } finally {
      setRefreshing(false);
    }
  }

  async function handleSend() {
    if (!draft.trim()) {
      return;
    }
    setSending(true);
    try {
      await sendMessage(draft);
      setDraft("");
    } catch (cause) {
      toast.error(cause instanceof Error ? cause.message : "发送失败");
    } finally {
      setSending(false);
    }
  }

  function handleKeyDown(event: React.KeyboardEvent<HTMLTextAreaElement>) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      void handleSend();
    }
  }

  return (
    <PageScaffold
      title="客户会话"
      subtitle="查看买家消息并人工回复"
      scroll={false}
      fill
      extra={
        <div className="flex items-center gap-2">
          <Select
            value={accountFilter || "__all__"}
            onValueChange={(value) =>
              setAccountFilter(value === "__all__" ? "" : value)
            }
          >
            <SelectTrigger className="w-[10rem]">
              <SelectValue placeholder="全部账号" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="__all__">全部账号</SelectItem>
              {accounts.map((account) => (
                <SelectItem key={account.account_id} value={account.account_id}>
                  {account.display_name || account.account_id}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Button
            size="icon"
            variant="ghost"
            aria-label="刷新"
            disabled={refreshing}
            onClick={() => void handleRefresh()}
          >
            <RefreshCw
              className={`size-4 ${refreshing ? "animate-spin" : ""}`}
              aria-hidden
            />
          </Button>
        </div>
      }
    >
      <div className="flex min-h-0 flex-1 overflow-hidden rounded-2xl border border-border bg-card">
        <aside className="flex w-72 shrink-0 flex-col border-r border-border bg-muted/20">
          <div className="border-b border-border px-3 py-2 text-[length:var(--text-xs)] text-muted-foreground">
            {filteredConversations.length} 个会话
          </div>
          <ScrollArea className="min-h-0 flex-1">
            {loading ? (
              <div className="flex justify-center py-8">
                <Loading />
              </div>
            ) : filteredConversations.length === 0 ? (
              <div className="space-y-3 px-4 py-8 text-center text-[length:var(--text-sm)] text-muted-foreground">
                <MessageSquare className="mx-auto size-8 opacity-40" aria-hidden />
                <p>暂无会话</p>
                <p className="text-[length:var(--text-xs)]">
                  请先在账号管理连接闲鱼账号，未读消息会自动同步到这里。
                </p>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => selectTab(managePath("accounts"))}
                >
                  前往账号管理
                </Button>
              </div>
            ) : (
              <div className="space-y-1 py-2">
                {groupedConversations.map((group) => {
                  const collapsed = collapsedAccountIds.has(group.accountId);
                  return (
                    <div key={group.accountId}>
                      <button
                        type="button"
                        className="flex w-full items-center gap-2 px-3 py-2 text-left transition-colors hover:bg-muted/50 active:scale-[0.99]"
                        aria-expanded={!collapsed}
                        onClick={() => toggleAccountGroup(group.accountId)}
                      >
                        {collapsed ? (
                          <ChevronRight
                            className="size-3.5 shrink-0 text-muted-foreground"
                            aria-hidden
                          />
                        ) : (
                          <ChevronDown
                            className="size-3.5 shrink-0 text-muted-foreground"
                            aria-hidden
                          />
                        )}
                        <ConversationAvatar
                          name={group.label}
                          src={group.avatarUrl}
                          size="sm"
                        />
                        <span className="min-w-0 flex-1 truncate text-[length:var(--text-xs)] font-medium text-muted-foreground">
                          {group.label}
                        </span>
                        <span className="shrink-0 tabular-nums text-[length:var(--text-xs)] text-muted-foreground/80">
                          {group.list.length}
                        </span>
                      </button>
                      {collapsed ? null : (
                        <ul className="pb-1">
                          {group.list.map((conversation) => {
                            const active = conversation.id === selectedId;
                            const preview = lastMessagePreview(
                              conversation.id,
                              messages,
                            );
                            const needsReply = conversationNeedsReply(
                              conversation.id,
                              messages,
                            );
                            const title =
                              conversation.peer_name?.trim() ||
                              conversation.peer_id ||
                              "未知联系人";
                            return (
                              <li key={conversation.id}>
                                <button
                                  type="button"
                                  className={cn(
                                    "flex w-full items-start gap-2.5 py-2.5 pl-9 pr-3 text-left transition-colors duration-150 ease-[cubic-bezier(0.23,1,0.32,1)] hover:bg-muted/60 active:scale-[0.99]",
                                    active && "bg-primary/10 hover:bg-primary/15",
                                  )}
                                  onClick={() =>
                                    selectConversation(conversation.id)
                                  }
                                >
                                  <ConversationAvatar name={title} size="md" />
                                  <div className="min-w-0 flex-1">
                                    <div className="flex items-start justify-between gap-2">
                                      <span className="truncate font-medium text-[length:var(--text-sm)]">
                                        {title}
                                      </span>
                                      {needsReply ? (
                                        <span className="mt-1 size-2 shrink-0 rounded-full bg-primary" />
                                      ) : null}
                                    </div>
                                    <p className="mt-0.5 truncate text-[length:var(--text-xs)] text-muted-foreground">
                                      {preview || "暂无消息"}
                                    </p>
                                    {conversation.item_title ||
                                    conversation.item_id ? (
                                      <p className="mt-0.5 truncate text-[length:var(--text-xs)] text-muted-foreground/80">
                                        {conversation.item_title
                                          ? conversation.item_title
                                          : `商品 ${conversation.item_id}`}
                                      </p>
                                    ) : null}
                                  </div>
                                </button>
                              </li>
                            );
                          })}
                        </ul>
                      )}
                    </div>
                  );
                })}
              </div>
            )}
          </ScrollArea>
        </aside>

        <section className="flex min-w-0 flex-1 flex-col">
          {!selectedConversation ? (
            <div className="flex flex-1 flex-col items-center justify-center gap-2 text-muted-foreground">
              <MessageSquare className="size-10 opacity-30" aria-hidden />
              <p className="text-[length:var(--text-sm)]">选择左侧会话查看消息</p>
            </div>
          ) : (
            <>
              <header className="flex items-center justify-between border-b border-border px-4 py-3">
                <div className="flex min-w-0 items-center gap-3">
                  <ConversationAvatar name={peerName} />
                  <div className="min-w-0">
                    <h2 className="truncate font-medium text-[length:var(--text-base)]">
                      {peerName}
                    </h2>
                    <p className="text-[length:var(--text-xs)] text-muted-foreground">
                      账号：{accountLabel}
                    </p>
                  </div>
                </div>
                <IconButton
                  label={infoOpen ? "收起客户信息" : "展开客户信息"}
                  onClick={() => setInfoOpen((open) => !open)}
                  className="shrink-0"
                >
                  {infoOpen ? (
                    <PanelRightClose className="size-4" aria-hidden />
                  ) : (
                    <PanelRightOpen className="size-4" aria-hidden />
                  )}
                </IconButton>
              </header>

              <ScrollArea className="min-h-0 flex-1 px-4 py-4">
                <div className="space-y-3">
                  {threadMessages.length === 0 ? (
                    <p className="py-8 text-center text-[length:var(--text-sm)] text-muted-foreground">
                      该会话还没有消息记录
                    </p>
                  ) : (
                    threadMessages.map((message) => {
                      const outbound = message.direction === "out";
                      return (
                        <div
                          key={message.id}
                          className={`flex ${outbound ? "justify-end" : "justify-start"}`}
                        >
                          <div
                            className={`max-w-[75%] rounded-2xl px-3 py-2 text-[length:var(--text-sm)] ${
                              outbound
                                ? "bg-primary text-primary-foreground"
                                : "bg-muted text-foreground"
                            }`}
                          >
                            <p className="whitespace-pre-wrap break-words">{message.content}</p>
                            <p
                              className={`mt-1 text-[length:var(--text-xs)] ${
                                outbound
                                  ? "text-primary-foreground/70"
                                  : "text-muted-foreground"
                              }`}
                            >
                              {formatMessageTime(message.created_at)}
                              {outbound && message.sender !== "human"
                                ? ` · ${message.sender}`
                                : ""}
                            </p>
                          </div>
                        </div>
                      );
                    })
                  )}
                  <div ref={threadEndRef} />
                </div>
              </ScrollArea>

              <footer className="border-t border-border p-3">
                {error ? (
                  <p className="mb-2 text-[length:var(--text-xs)] text-destructive">
                    {error}
                  </p>
                ) : null}
                <div className="flex gap-2">
                  <Textarea
                    value={draft}
                    onChange={(event) => setDraft(event.target.value)}
                    onKeyDown={handleKeyDown}
                    placeholder="输入回复，Enter 发送，Shift+Enter 换行"
                    rows={2}
                    className="min-h-[3rem] resize-none"
                  />
                  <AsyncButton
                    loading={sending}
                    disabled={!draft.trim()}
                    onClick={() => handleSend()}
                    className="shrink-0 self-end"
                  >
                    <Send className="size-4" aria-hidden />
                    发送
                  </AsyncButton>
                </div>
              </footer>
            </>
          )}
        </section>

        {infoOpen ? (
          <aside className="flex w-72 shrink-0 flex-col border-l border-border bg-muted/20">
            <div className="border-b border-border px-3 py-2 text-[length:var(--text-xs)] text-muted-foreground">
              客户信息
            </div>
            <ScrollArea className="min-h-0 flex-1">
              <div className="space-y-3 p-3">
                {!selectedConversation ? (
                  <p className="py-8 text-center text-[length:var(--text-sm)] text-muted-foreground">
                    选择左侧会话查看客户信息
                  </p>
                ) : (
                  <>
                    <InfoSection icon={User} title="买家资料">
                      <InfoRow label="昵称" value={peerName} />
                      <InfoRow
                        label="买家 ID"
                        value={selectedConversation.peer_id || undefined}
                        mono
                      />
                      <InfoRow label="鱼塘昵称" value={buyerFishNick} />
                      <InfoRow label="买家昵称" value={buyerNick} />
                      <InfoRow label="所属账号" value={accountLabel} />
                    </InfoSection>

                    <InfoSection icon={Package} title="商品信息">
                      <InfoRow
                        label="商品"
                        value={headTitle || selectedConversation.item_title || undefined}
                      />
                      <InfoRow label="价格" value={headPrice || itemPrice} />
                      {headImage ? (
                        <img
                          src={headImage}
                          alt="商品图"
                          className="mt-1 h-24 w-full rounded-lg border border-border object-cover"
                        />
                      ) : null}
                      <InfoRow
                        label="商品 ID"
                        value={selectedConversation.item_id || undefined}
                        mono
                      />
                    </InfoSection>

                    <InfoSection icon={MessageSquare} title="会话概览">
                      <InfoRow label="消息数" value={String(threadMessages.length)} />
                      <InfoRow label="最后消息" value={lastMessageTime} />
                      <InfoRow label="待回复" value={needsReply ? "是" : "否"} />
                      <InfoRow label="账号" value={accountLabel} />
                    </InfoSection>

                    <InfoSection
                      icon={ShoppingCart}
                      title={`订单信息${buyerOrders.length ? ` (${buyerOrders.length})` : ""}`}
                    >
                      {ordersLoading ? (
                        <div className="flex justify-center py-4">
                          <Loading />
                        </div>
                      ) : ordersError ? (
                        <p className="text-[length:var(--text-xs)] text-destructive">
                          {ordersError}
                        </p>
                      ) : buyerOrders.length === 0 ? (
                        <p className="text-[length:var(--text-xs)] text-muted-foreground">
                          暂无订单
                        </p>
                      ) : (
                        <div className="space-y-2">
                          {buyerOrders.map((order) => (
                            <div
                              key={order.order_no}
                              className="rounded-lg border border-border/70 bg-muted/30 p-2"
                            >
                              <div className="flex items-center justify-between gap-2">
                                <span className="truncate font-mono text-[length:var(--text-xs)]">
                                  {order.order_no}
                                </span>
                                <span className="shrink-0 text-[length:var(--text-xs)]">
                                  {ORDER_STATUS_LABELS[order.status] ?? order.status}
                                </span>
                              </div>
                              <div className="mt-1 flex items-center justify-between gap-2 text-[length:var(--text-xs)] text-muted-foreground">
                                <span className="truncate">
                                  {order.item_title || order.item_id}
                                  {order.spec_value ? ` · ${order.spec_value}` : ""}
                                </span>
                                <span className="shrink-0">
                                  {formatAmount(order.amount)} × {order.quantity}
                                </span>
                              </div>
                              {order.placed_at ? (
                                <p className="mt-1 text-[length:var(--text-xs)] text-muted-foreground/80">
                                  {formatDateTime(order.placed_at)}
                                </p>
                              ) : null}
                            </div>
                          ))}
                        </div>
                      )}
                    </InfoSection>
                  </>
                )}
              </div>
            </ScrollArea>
          </aside>
        ) : null}
      </div>
    </PageScaffold>
  );
}

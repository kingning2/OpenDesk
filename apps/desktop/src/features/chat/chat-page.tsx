/**
 * 客户会话收件箱 — 会话列表、消息记录与人工发送。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */

import { useEffect, useMemo, useRef, useState } from "react";
import {
  AsyncButton,
  Button,
  Loading,
  PageScaffold,
  ScrollArea,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Textarea,
  toast,
} from "@desk/ui";
import { MessageSquare, RefreshCw, Send } from "@desk/ui/icons";
import { managePath } from "@desk/platform/compile";
import { accountList, type XianyuAccount } from "@desk/platform/ipc/account";
import { useWorkspaceNav } from "../../app/use-workspace-tabs";
import {
  conversationNeedsReply,
  lastMessagePreview,
  useChannelInbox,
} from "./use-channel-inbox";

const OWNER_ID = 1;

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
  const threadEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    void accountList(OWNER_ID)
      .then(setAccounts)
      .catch(() => setAccounts([]));
  }, []);

  useEffect(() => {
    threadEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [threadMessages, selectedId]);

  const accountLabelById = useMemo(() => {
    const map = new Map<string, string>();
    for (const account of accounts) {
      map.set(
        account.account_id,
        account.display_name || account.account_id,
      );
    }
    return map;
  }, [accounts]);

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
      containerPadding="none"
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
      <div className="flex min-h-0 flex-1 border-t border-border">
        <aside className="flex w-72 shrink-0 flex-col border-r border-border bg-muted/20">
          <div className="border-b border-border px-3 py-2 text-xs text-muted-foreground">
            {filteredConversations.length} 个会话
          </div>
          <ScrollArea className="min-h-0 flex-1">
            {loading ? (
              <div className="flex justify-center py-8">
                <Loading />
              </div>
            ) : filteredConversations.length === 0 ? (
              <div className="space-y-3 px-4 py-8 text-center text-sm text-muted-foreground">
                <MessageSquare className="mx-auto size-8 opacity-40" aria-hidden />
                <p>暂无会话</p>
                <p className="text-xs">
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
              <ul className="divide-y divide-border">
                {filteredConversations.map((conversation) => {
                  const active = conversation.id === selectedId;
                  const preview = lastMessagePreview(conversation.id, messages);
                  const needsReply = conversationNeedsReply(conversation.id, messages);
                  const title =
                    conversation.peer_name?.trim() ||
                    conversation.peer_id ||
                    "未知联系人";
                  return (
                    <li key={conversation.id}>
                      <button
                        type="button"
                        className={`w-full px-3 py-3 text-left transition-colors hover:bg-muted/60 ${
                          active ? "bg-primary/10" : ""
                        }`}
                        onClick={() => selectConversation(conversation.id)}
                      >
                        <div className="flex items-start justify-between gap-2">
                          <span className="truncate font-medium text-sm">{title}</span>
                          {needsReply ? (
                            <span className="mt-1 size-2 shrink-0 rounded-full bg-primary" />
                          ) : null}
                        </div>
                        <p className="mt-1 truncate text-xs text-muted-foreground">
                          {preview || "暂无消息"}
                        </p>
                        <p className="mt-1 text-[10px] text-muted-foreground/80">
                          {accountLabelById.get(conversation.account_id) ??
                            conversation.account_id}
                          {conversation.item_title
                            ? ` · ${conversation.item_title}`
                            : conversation.item_id
                              ? ` · 商品 ${conversation.item_id}`
                              : ""}
                        </p>
                      </button>
                    </li>
                  );
                })}
              </ul>
            )}
          </ScrollArea>
        </aside>

        <section className="flex min-w-0 flex-1 flex-col">
          {!selectedConversation ? (
            <div className="flex flex-1 flex-col items-center justify-center gap-2 text-muted-foreground">
              <MessageSquare className="size-10 opacity-30" aria-hidden />
              <p className="text-sm">选择左侧会话查看消息</p>
            </div>
          ) : (
            <>
              <header className="border-b border-border px-4 py-3">
                <h2 className="font-medium">
                  {selectedConversation.peer_name?.trim() ||
                    selectedConversation.peer_id ||
                    "未知联系人"}
                </h2>
                <p className="text-xs text-muted-foreground">
                  账号：
                  {accountLabelById.get(selectedConversation.account_id) ??
                    selectedConversation.account_id}
                </p>
              </header>

              <ScrollArea className="min-h-0 flex-1 px-4 py-4">
                <div className="space-y-3">
                  {threadMessages.length === 0 ? (
                    <p className="py-8 text-center text-sm text-muted-foreground">
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
                            className={`max-w-[75%] rounded-2xl px-3 py-2 text-sm ${
                              outbound
                                ? "bg-primary text-primary-foreground"
                                : "bg-muted text-foreground"
                            }`}
                          >
                            <p className="whitespace-pre-wrap break-words">{message.content}</p>
                            <p
                              className={`mt-1 text-[10px] ${
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
                  <p className="mb-2 text-xs text-destructive">{error}</p>
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
      </div>
    </PageScaffold>
  );
}

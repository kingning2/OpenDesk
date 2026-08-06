/**
 * Chat Feature 页 — 左侧会话列表 + 右侧 AI 对话（流式逐字输出，多会话持久化）。
 *
 * 会话支持新建/切换/重命名/删除，历史从 chat.db 恢复；流式事件按 session 过滤，
 * 切走的会话继续在后台流式并落库。
 *
 * @author coisini
 */

import { useState, type FormEvent } from "react";
import {
  Bubble,
  BubbleContent,
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  IconButton,
  Input,
  LoadingState,
  Message,
  MessageContent,
  MessageScroller,
  MessageScrollerContent,
  MessageScrollerItem,
  MessageScrollerProvider,
  MessageScrollerViewport,
  MessageScrollerButton,
  PageScaffold,
  WorkspaceSplit,
  WorkspaceSplitPane,
  WorkspaceSplitTitle,
  WorkspaceSplitToolbar,
  cn,
} from "@desk/ui";
import {
  ChevronRight,
  Pencil,
  Plus,
  Search,
  Sparkles,
  Trash2,
} from "@desk/ui/icons";
import type { ChatSession } from "@desk/platform/ipc/chat";

import { useT } from "../../i18n";
import { MarkdownContent } from "./markdown";
import { useChat, type ChatMessageView, type ToolStep } from "./use-chat";

/** 「思考中…」三点跳动动画。 */
function ThinkingDots() {
  const t = useT();
  return (
    <span
      className="inline-flex items-center gap-1"
      role="status"
      aria-label={t("chat.thinking")}
    >
      {[0, 1, 2].map((index) => (
        <span
          key={index}
          className="size-1.5 animate-bounce rounded-full bg-current opacity-60"
          style={{ animationDelay: `${index * 150}ms` }}
        />
      ))}
    </span>
  );
}

/**
 * 可折叠的「思考过程」块：有推理内容时默认展开，让用户实时看到思考进行到哪一步；
 * 用户可手动折叠/展开（折叠后保持折叠）。
 */
function ThinkingBlock({ message }: { message: ChatMessageView }) {
  const t = useT();
  const [userClosed, setUserClosed] = useState(false);
  const thinking = message.thinking?.trim();
  if (!thinking) {
    return null;
  }

  return (
    <details
      className="group/thinking max-w-[80%]"
      open={!userClosed}
      onToggle={(event) => setUserClosed(!event.currentTarget.open)}
    >
      <summary className="flex cursor-pointer list-none items-center gap-1.5 text-xs font-medium text-muted-foreground [@media(hover:hover)_and_(pointer:fine)]:hover:text-foreground">
        <Sparkles className="size-3.5 shrink-0" aria-hidden />
        <span>{t("chat.thinkingDetails")}</span>
        <ChevronRight className="size-3 transition-transform duration-200 group-open/thinking:rotate-90" aria-hidden />
      </summary>
      <p className="mt-2 whitespace-pre-wrap rounded-[var(--radius-md)] border border-border bg-muted/40 px-3 py-2 text-[length:var(--text-sm)] leading-relaxed text-muted-foreground">
        {thinking}
      </p>
    </details>
  );
}

/**
 * 工具调用步骤块：小号等宽展示工具名 + 参数 + 结果/错误，复用 muted 样式。
 * 整块可折叠（默认展开，让用户实时看到 AI 查了哪些库；手动折叠后保持折叠）。
 */
function ToolSteps({ tools }: { tools: ToolStep[] }) {
  const t = useT();
  const [userClosed, setUserClosed] = useState(false);
  if (tools.length === 0) {
    return null;
  }

  return (
    <details
      className="group/tools mb-2 max-w-[80%]"
      open={!userClosed}
      onToggle={(event) => setUserClosed(!event.currentTarget.open)}
    >
      <summary className="flex cursor-pointer list-none items-center gap-1.5 text-xs font-medium text-muted-foreground [@media(hover:hover)_and_(pointer:fine)]:hover:text-foreground">
        <Search className="size-3.5 shrink-0" aria-hidden />
        <span>{t("chat.toolCalls")}</span>
        <span className="rounded-full bg-muted px-1.5 text-[10px] leading-4 text-muted-foreground">
          {tools.length}
        </span>
        <ChevronRight
          className="size-3 transition-transform duration-200 group-open/tools:rotate-90"
          aria-hidden
        />
      </summary>
      <div className="mt-2 flex flex-col gap-1.5">
        {tools.map((tool, index) => (
          <div
            key={index}
            className="rounded-[var(--radius-md)] border border-border bg-muted/40 px-3 py-2"
          >
            <div className="flex items-center gap-1.5 font-mono text-[length:var(--text-xs)] font-medium">
              <span className="truncate text-foreground">{tool.name}</span>
              <span
                className={
                  tool.ok
                    ? "text-emerald-600 dark:text-emerald-400"
                    : "text-red-500"
                }
              >
                {tool.ok ? t("chat.toolOk") : t("chat.toolFailed")}
              </span>
            </div>
            {tool.arguments ? (
              <div className="mt-1 whitespace-pre-wrap break-all font-mono text-[length:var(--text-xs)] leading-relaxed text-muted-foreground">
                {tool.arguments}
              </div>
            ) : null}
            {tool.result ? (
              <div className="mt-1 whitespace-pre-wrap break-all font-mono text-[length:var(--text-xs)] leading-relaxed text-muted-foreground">
                {tool.result}
              </div>
            ) : null}
          </div>
        ))}
      </div>
    </details>
  );
}

function ChatBubble({ message }: { message: ChatMessageView }) {
  const isUser = message.role === "user";

  return (
    <Message align={isUser ? "end" : "start"}>
      <MessageContent>
        {!isUser ? <ThinkingBlock message={message} /> : null}
        {!isUser && message.tools && message.tools.length > 0 ? (
          <ToolSteps tools={message.tools} />
        ) : null}

        <Bubble
          variant={isUser ? "default" : "muted"}
          align={isUser ? "end" : "start"}
        >
          <BubbleContent
            className={cn(
              message.error && "border-red-500/40 text-red-600 dark:text-red-300",
            )}
          >
            {message.error ? (
              <div className="whitespace-pre-wrap">{message.content}</div>
            ) : (
              <MarkdownContent content={message.content} />
            )}
            {message.streaming ? (
              message.content ? (
                <span className="ml-0.5 inline-block h-4 w-0.5 animate-pulse bg-current" />
              ) : (
                <ThinkingDots />
              )
            ) : null}
          </BubbleContent>
        </Bubble>
      </MessageContent>
    </Message>
  );
}

/** 会话行：点击切换，悬停显示重命名/删除。 */
function SessionRow({
  session,
  active,
  onSelect,
  onRename,
  onDelete,
}: {
  session: ChatSession;
  active: boolean;
  onSelect: () => void;
  onRename: () => void;
  onDelete: () => void;
}) {
  const t = useT();
  return (
    <div
      className={cn(
        "group flex w-full items-center gap-1 border-b border-border/50 pr-1.5 transition-colors",
        "duration-150 ease-[cubic-bezier(0.23,1,0.32,1)]",
        active
          ? "bg-primary/10"
          : "[@media(hover:hover)_and_(pointer:fine)]:hover:bg-muted/40",
      )}
    >
      <button
        type="button"
        className="min-w-0 flex-1 px-3 py-2.5 text-left"
        onClick={onSelect}
      >
        <span
          className={cn(
            "block truncate text-[length:var(--text-sm)] text-foreground",
            active && "font-semibold",
          )}
        >
          {session.title || t("chat.sessions.untitled")}
        </span>
        <span className="mt-0.5 block text-[10px] text-muted-foreground">
          {session.last_message_at
            ? formatSessionTime(session.last_message_at)
            : ""}
          {session.message_count > 0
            ? ` · ${t("chat.sessions.messages", { count: session.message_count })}`
            : ""}
        </span>
      </button>
      <div className="flex shrink-0 items-center gap-0.5 opacity-0 transition-opacity group-hover:opacity-100 group-focus-within:opacity-100">
        <IconButton
          label={t("chat.sessions.rename")}
          className="size-6"
          onClick={onRename}
        >
          <Pencil className="size-3" />
        </IconButton>
        <IconButton
          label={t("chat.sessions.delete")}
          className="size-6"
          onClick={onDelete}
        >
          <Trash2 className="size-3" />
        </IconButton>
      </div>
    </div>
  );
}

/** 会话列表行时间：今天显示时分，往年显示完整日期。 */
function formatSessionTime(timestamp: number): string {
  const date = new Date(timestamp);
  const now = new Date();
  if (date.toDateString() === now.toDateString()) {
    return new Intl.DateTimeFormat(undefined, {
      hour: "2-digit",
      minute: "2-digit",
    }).format(date);
  }
  if (date.getFullYear() === now.getFullYear()) {
    return new Intl.DateTimeFormat(undefined, {
      month: "2-digit",
      day: "2-digit",
    }).format(date);
  }
  return new Intl.DateTimeFormat(undefined, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).format(date);
}

/**
 * Chat 页面。
 *
 * @author coisini
 *
 * @returns 页面节点
 */
export function ChatPage() {
  const t = useT();
  const {
    sessions,
    activeSessionId,
    messages,
    sending,
    error,
    loading,
    clearError,
    send,
    switchSession,
    createSession,
    renameSession,
    deleteSession,
  } = useChat();
  const [input, setInput] = useState("");
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");
  const [deleteTarget, setDeleteTarget] = useState<ChatSession | null>(null);

  const notConfigured = /not configured/i.test(error);
  const activeTitle =
    sessions.find((item) => item.id === activeSessionId)?.title || "";

  const onSubmit = (event: FormEvent) => {
    event.preventDefault();
    const text = input.trim();
    if (!text || sending) {
      return;
    }
    void send(text);
    setInput("");
  };

  const submitRename = (event: FormEvent) => {
    event.preventDefault();
    if (renamingId && renameValue.trim()) {
      void renameSession(renamingId, renameValue.trim());
    }
    setRenamingId(null);
    setRenameValue("");
  };

  if (loading) {
    return (
      <PageScaffold fill containerPadding="none">
        <LoadingState label={t("chat.sessions.loading")} className="min-h-0 flex-1" />
      </PageScaffold>
    );
  }

  return (
    <PageScaffold fill containerPadding="none">
      <WorkspaceSplit
        className="min-h-0 flex-1 border-0"
        defaultStartWidth={260}
        minStartWidth={200}
        maxStartWidth={360}
      >
        <WorkspaceSplitPane
          side="start"
          header={
            <WorkspaceSplitToolbar className="justify-between px-3 py-2">
              <WorkspaceSplitTitle className="text-xs">
                {t("chat.sessions.title")}
              </WorkspaceSplitTitle>
              <Button
                size="sm"
                variant="ghost"
                className="size-7 p-0"
                title={t("chat.sessions.new")}
                onClick={() => void createSession()}
              >
                <Plus className="size-3.5" />
              </Button>
            </WorkspaceSplitToolbar>
          }
        >
          <div className="flex flex-col">
            {sessions.length === 0 ? (
              <p className="px-3 py-6 text-center text-[length:var(--text-xs)] text-muted-foreground">
                {t("chat.sessions.empty")}
              </p>
            ) : (
              sessions.map((session) => (
                <SessionRow
                  key={session.id}
                  session={session}
                  active={session.id === activeSessionId}
                  onSelect={() => void switchSession(session.id)}
                  onRename={() => {
                    setRenamingId(session.id);
                    setRenameValue(session.title || "");
                  }}
                  onDelete={() => setDeleteTarget(session)}
                />
              ))
            )}
          </div>
        </WorkspaceSplitPane>

        <WorkspaceSplitPane scroll={false}>
          <div className="flex h-full min-h-0 flex-col">
            <WorkspaceSplitToolbar className="justify-between px-4 py-2">
              <WorkspaceSplitTitle className="truncate text-sm">
                {activeTitle || t("chat.title")}
              </WorkspaceSplitTitle>
            </WorkspaceSplitToolbar>

            <MessageScrollerProvider autoScroll defaultScrollPosition="end">
              <MessageScroller className="min-h-0 flex-1">
                <MessageScrollerViewport>
                  <MessageScrollerContent>
                    {messages.length === 0 ? (
                      <p className="py-16 text-center text-[length:var(--text-sm)] text-muted-foreground">
                        {t("chat.empty")}
                      </p>
                    ) : (
                      messages.map((message, index) => (
                        <MessageScrollerItem
                          key={message.id}
                          messageId={message.id}
                          scrollAnchor={index === messages.length - 1}
                        >
                          <ChatBubble message={message} />
                        </MessageScrollerItem>
                      ))
                    )}
                  </MessageScrollerContent>
                </MessageScrollerViewport>
                <MessageScrollerButton direction="end" />
              </MessageScroller>
            </MessageScrollerProvider>

            {error ? (
              <div className="flex items-start justify-between gap-2 border-t border-border/50 px-4 py-2">
                <p className="text-[length:var(--text-sm)] text-red-600 dark:text-red-300">
                  {notConfigured ? t("chat.errorNotConfigured") : error}
                </p>
                <button
                  type="button"
                  onClick={clearError}
                  aria-label={t("chat.dismiss")}
                  className="text-[length:var(--text-sm)] text-muted-foreground"
                >
                  ×
                </button>
              </div>
            ) : null}

            <form onSubmit={onSubmit} className="flex items-center gap-2 p-4 pt-2">
              <Input
                value={input}
                onChange={(event) => setInput(event.target.value)}
                placeholder={t("chat.input.placeholder")}
                disabled={sending}
                className="flex-1"
              />
              <Button type="submit" disabled={sending || !input.trim()}>
                {sending ? t("chat.sending") : t("chat.send")}
              </Button>
            </form>
          </div>
        </WorkspaceSplitPane>
      </WorkspaceSplit>

      {renamingId ? (
        <Dialog
          open
          onOpenChange={(open) => {
            if (!open) {
              setRenamingId(null);
            }
          }}
        >
          <DialogContent className="max-w-sm">
            <DialogHeader>
              <DialogTitle>{t("chat.sessions.renameTitle")}</DialogTitle>
            </DialogHeader>
            <form onSubmit={submitRename}>
              <Input
                autoFocus
                value={renameValue}
                onChange={(event) => setRenameValue(event.target.value)}
                placeholder={t("chat.sessions.renamePlaceholder")}
              />
              <DialogFooter className="mt-4">
                <Button variant="outline" onClick={() => setRenamingId(null)}>
                  {t("chat.sessions.cancel")}
                </Button>
                <Button type="submit" disabled={!renameValue.trim()}>
                  {t("chat.sessions.rename")}
                </Button>
              </DialogFooter>
            </form>
          </DialogContent>
        </Dialog>
      ) : null}

      {deleteTarget ? (
        <Dialog
          open
          onOpenChange={(open) => {
            if (!open) {
              setDeleteTarget(null);
            }
          }}
        >
          <DialogContent className="max-w-sm">
            <DialogHeader>
              <DialogTitle>{t("chat.sessions.deleteTitle")}</DialogTitle>
              <DialogDescription>
                {t("chat.sessions.deleteConfirm", {
                  title:
                    deleteTarget.title || t("chat.sessions.untitled"),
                })}
              </DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <Button variant="outline" onClick={() => setDeleteTarget(null)}>
                {t("chat.sessions.cancel")}
              </Button>
              <Button
                className="bg-red-600 text-white [@media(hover:hover)_and_(pointer:fine)]:hover:bg-red-500"
                onClick={() => {
                  const id = deleteTarget.id;
                  setDeleteTarget(null);
                  void deleteSession(id);
                }}
              >
                {t("chat.sessions.delete")}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      ) : null}
    </PageScaffold>
  );
}

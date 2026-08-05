/**
 * Chat Feature 页 — 与 AI 对话（流式逐字输出，会话内多轮）。
 *
 * 展示层复用 shadcn/ui chat 组件（Message / Bubble / MessageScroller），
 * 状态层沿用 `useChat` hook；支持「新窗口打开」并行独立会话窗口。
 *
 * @author coisini
 */

import { useState, type FormEvent } from "react";
import {
  Bubble,
  BubbleContent,
  Button,
  Input,
  Message,
  MessageContent,
  MessageScroller,
  MessageScrollerContent,
  MessageScrollerItem,
  MessageScrollerProvider,
  MessageScrollerViewport,
  MessageScrollerButton,
  PageScaffold,
  cn,
} from "@desk/ui";
import { ChevronRight, Search, Sparkles } from "@desk/ui/icons";

import { useT } from "../../i18n";
import { useChat, type ChatMessage, type ToolStep } from "./use-chat";

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
function ThinkingBlock({ message }: { message: ChatMessage }) {
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
 * 渲染在思考过程与正文之间，让用户看到 AI 查了哪些库。
 */
function ToolSteps({ tools }: { tools: ToolStep[] }) {
  const t = useT();
  if (tools.length === 0) {
    return null;
  }

  return (
    <div className="mb-2 flex max-w-[80%] flex-col gap-1.5">
      {tools.map((tool, index) => (
        <div
          key={index}
          className="rounded-[var(--radius-md)] border border-border bg-muted/40 px-3 py-2"
        >
          <div className="flex items-center gap-1.5 font-mono text-[length:var(--text-xs)] font-medium">
            <Search className="size-3 shrink-0 text-muted-foreground" aria-hidden />
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
  );
}

function ChatBubble({ message }: { message: ChatMessage }) {
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
              "whitespace-pre-wrap",
              message.error && "border-red-500/40 text-red-600 dark:text-red-300",
            )}
          >
            {message.content}
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

/**
 * Chat 页面。
 *
 * @author coisini
 *
 * @returns 页面节点
 */
export function ChatPage() {
  const t = useT();
  const { messages, sending, error, clearError, send } = useChat();
  const [input, setInput] = useState("");

  const notConfigured = /not configured/i.test(error);

  const onSubmit = (event: FormEvent) => {
    event.preventDefault();
    const text = input.trim();
    if (!text || sending) {
      return;
    }
    void send(text);
    setInput("");
  };

  return (
    <PageScaffold fill containerPadding="md">
      <h1 className="text-[length:var(--text-lg)] font-semibold">{t("chat.title")}</h1>

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
        <div className="flex items-start justify-between gap-2 rounded-[var(--radius-md)] border border-red-500/40 bg-red-500/10 px-3 py-2">
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

      <form onSubmit={onSubmit} className="flex items-center gap-2">
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
    </PageScaffold>
  );
}

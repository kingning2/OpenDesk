/**
 * Help Feature 页 — 一问一答的系统导航问答。
 *
 * 每次问答相互独立：发送新问题时清空上一轮，重新开始；不保留历史、不用长期记忆。
 * 流式逐字输出；动作工具（navigate_page / open_settings）渲染成按钮，用户点击才跳转。
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
  PageScaffold,
  WorkspaceSplitTitle,
  WorkspaceSplitToolbar,
  cn,
} from "@desk/ui";
import { ChevronRight, Sparkles } from "@desk/ui/icons";

import { useT } from "../../i18n";
import { MarkdownContent } from "../chat/markdown";
import { SuggestedActions } from "../chat/suggested-actions";
import { useHelp, type HelpReply } from "./use-help";

/** 可折叠的「思考过程」块：有推理内容时展示，用户可手动折叠。 */
function Thinking({ reply }: { reply: HelpReply }) {
  const t = useT();
  const thinking = reply.thinking.trim();
  if (!thinking) {
    return null;
  }

  return (
    <details className="group/thinking max-w-[80%]" open>
      <summary className="flex cursor-pointer list-none items-center gap-1.5 text-xs font-medium text-muted-foreground [@media(hover:hover)_and_(pointer:fine)]:hover:text-foreground">
        <Sparkles className="size-3.5 shrink-0" aria-hidden />
        <span>{t("help.thinkingDetails")}</span>
        <ChevronRight
          className="size-3 transition-transform duration-200 group-open/thinking:rotate-90"
          aria-hidden
        />
      </summary>
      <p className="mt-2 whitespace-pre-wrap rounded-[var(--radius-md)] border border-border bg-muted/40 px-3 py-2 text-[length:var(--text-sm)] leading-relaxed text-muted-foreground">
        {thinking}
      </p>
    </details>
  );
}

/** 「思考中…」三点跳动动画。 */
function ThinkingDots() {
  const t = useT();
  return (
    <span
      className="inline-flex items-center gap-1"
      role="status"
      aria-label={t("help.thinking")}
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
 * Help 页面。
 *
 * @author coisini
 *
 * @returns 页面节点
 */
export function HelpPage() {
  const t = useT();
  const { reply, sending, error, send, clearError } = useHelp();
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
    <PageScaffold fill containerPadding="none">
      <div className="flex h-full min-h-0 flex-col">
        <WorkspaceSplitToolbar className="px-4 py-2">
          <WorkspaceSplitTitle className="text-sm">{t("help.title")}</WorkspaceSplitTitle>
        </WorkspaceSplitToolbar>

        <MessageScrollerProvider autoScroll defaultScrollPosition="end">
          <MessageScroller className="min-h-0 flex-1">
            <MessageScrollerViewport>
              <MessageScrollerContent>
                {!reply ? (
                  <p className="px-6 py-16 text-center text-[length:var(--text-sm)] text-muted-foreground">
                    {t("help.empty")}
                  </p>
                ) : (
                  <>
                    <MessageScrollerItem messageId="help-question">
                      <Message align="end">
                        <MessageContent>
                          <Bubble variant="default" align="end">
                            <BubbleContent>{reply.question}</BubbleContent>
                          </Bubble>
                        </MessageContent>
                      </Message>
                    </MessageScrollerItem>
                    <MessageScrollerItem
                      messageId="help-answer"
                      scrollAnchor={reply.streaming}
                    >
                      <Message align="start">
                        <MessageContent>
                          <Thinking reply={reply} />
                          <SuggestedActions tools={reply.tools} />
                          <Bubble variant="muted" align="start">
                            <BubbleContent
                              className={cn(
                                reply.error &&
                                  "border-red-500/40 text-red-600 dark:text-red-300",
                              )}
                            >
                              {reply.error ? (
                                <div className="whitespace-pre-wrap">{reply.content}</div>
                              ) : (
                                <MarkdownContent content={reply.content} />
                              )}
                              {reply.streaming ? (
                                reply.content ? (
                                  <span className="ml-0.5 inline-block h-4 w-0.5 animate-pulse bg-current" />
                                ) : (
                                  <ThinkingDots />
                                )
                              ) : null}
                            </BubbleContent>
                          </Bubble>
                        </MessageContent>
                      </Message>
                    </MessageScrollerItem>
                  </>
                )}
              </MessageScrollerContent>
            </MessageScrollerViewport>
          </MessageScroller>
        </MessageScrollerProvider>

        {error ? (
          <div className="flex items-start justify-between gap-2 border-t border-border/50 px-4 py-2">
            <p className="text-[length:var(--text-sm)] text-red-600 dark:text-red-300">
              {notConfigured ? t("help.errorNotConfigured") : error}
            </p>
            <button
              type="button"
              onClick={clearError}
              aria-label={t("help.dismiss")}
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
            placeholder={t("help.input.placeholder")}
            disabled={sending}
            className="flex-1"
          />
          <Button type="submit" disabled={sending || !input.trim()}>
            {sending ? t("help.sending") : t("help.send")}
          </Button>
        </form>
      </div>
    </PageScaffold>
  );
}

import { useEffect, useRef, useState } from "react";
import { AsyncButton, IconButton, ScrollArea, Textarea, toast } from "@desk/ui";
import { MessageSquare, PanelRightClose, PanelRightOpen, Send } from "@desk/ui/icons";
import type { ChannelConversation, ChannelMessage } from "@desk/contracts";
import { ConversationAvatar } from "./conversation-avatar";
import { formatChatTime } from "./format";

export interface ThreadPanelProps {
  selectedConversation: ChannelConversation | null;
  peerName: string;
  accountLabel: string;
  threadMessages: ChannelMessage[];
  error: string | null;
  infoOpen: boolean;
  onToggleInfo: () => void;
  onSend: (text: string) => Promise<void>;
}

/** 会话消息区（中栏）— 消息记录 + 输入框。 */
export function ThreadPanel({
  selectedConversation,
  peerName,
  accountLabel,
  threadMessages,
  error,
  infoOpen,
  onToggleInfo,
  onSend,
}: ThreadPanelProps) {
  const [draft, setDraft] = useState("");
  const [sending, setSending] = useState(false);
  const threadEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    threadEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [threadMessages]);

  async function handleSend() {
    if (!draft.trim()) {
      return;
    }
    setSending(true);
    try {
      await onSend(draft);
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
              onClick={onToggleInfo}
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
                            outbound ? "text-primary-foreground/70" : "text-muted-foreground"
                          }`}
                        >
                          {formatChatTime(message.created_at)}
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
              <p className="mb-2 text-[length:var(--text-xs)] text-destructive">{error}</p>
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
  );
}

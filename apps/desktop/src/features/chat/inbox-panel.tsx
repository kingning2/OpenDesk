import { Button, Loading, ScrollArea } from "@desk/ui";
import { ChevronDown, ChevronRight, MessageSquare } from "@desk/ui/icons";
import type { ChannelConversation, ChannelMessage } from "@desk/contracts";
import { conversationNeedsReply, lastMessagePreview } from "./use-channel-inbox";
import { ConversationAvatar } from "./conversation-avatar";

export interface InboxGroup {
  accountId: string;
  label: string;
  avatarUrl: string;
  list: ChannelConversation[];
}

export interface InboxPanelProps {
  loading: boolean;
  conversationsCount: number;
  groups: InboxGroup[];
  messages: ChannelMessage[];
  selectedId: string | null;
  collapsedAccountIds: Set<string>;
  onToggleGroup: (accountId: string) => void;
  onSelectConversation: (id: string) => void;
  onGoToAccounts: () => void;
}

/** 会话收件箱（左栏）— 按账号分组的会话列表。 */
export function InboxPanel({
  loading,
  conversationsCount,
  groups,
  messages,
  selectedId,
  collapsedAccountIds,
  onToggleGroup,
  onSelectConversation,
  onGoToAccounts,
}: InboxPanelProps) {
  return (
    <aside className="flex w-72 shrink-0 flex-col border-r border-border bg-muted/20">
      <div className="border-b border-border px-3 py-2 text-[length:var(--text-xs)] text-muted-foreground">
        {conversationsCount} 个会话
      </div>
      <ScrollArea className="min-h-0 flex-1">
        {loading ? (
          <div className="flex justify-center py-8">
            <Loading />
          </div>
        ) : groups.length === 0 ? (
          <div className="space-y-3 px-4 py-8 text-center text-[length:var(--text-sm)] text-muted-foreground">
            <MessageSquare className="mx-auto size-8 opacity-40" aria-hidden />
            <p>暂无会话</p>
            <p className="text-[length:var(--text-xs)]">
              请先在账号管理连接闲鱼账号，未读消息会自动同步到这里。
            </p>
            <Button size="sm" variant="outline" onClick={onGoToAccounts}>
              前往账号管理
            </Button>
          </div>
        ) : (
          <div className="space-y-1 py-2">
            {groups.map((group) => {
              const collapsed = collapsedAccountIds.has(group.accountId);
              return (
                <div key={group.accountId}>
                  <button
                    type="button"
                    className="flex w-full items-center gap-2 px-3 py-2 text-left transition-colors hover:bg-muted/50 active:scale-[0.99]"
                    aria-expanded={!collapsed}
                    onClick={() => onToggleGroup(group.accountId)}
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
                    <ConversationAvatar name={group.label} src={group.avatarUrl} size="sm" />
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
                              className={`flex w-full items-start gap-2.5 py-2.5 pl-9 pr-3 text-left transition-colors duration-150 ease-[cubic-bezier(0.23,1,0.32,1)] hover:bg-muted/60 active:scale-[0.99] ${
                                active ? "bg-primary/10 hover:bg-primary/15" : ""
                              }`}
                              onClick={() => onSelectConversation(conversation.id)}
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
                                {conversation.item_title || conversation.item_id ? (
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
  );
}

import { Loading, ScrollArea } from "@desk/ui";
import { MessageSquare, Package, ShoppingCart, User } from "@desk/ui/icons";
import { formatAmount } from "@desk/utils";
import type { ChannelConversation, ChannelMessage } from "@desk/contracts";
import { ORDER_STATUS_LABELS } from "@feature/component/order-status";
import type { Order } from "@desk/platform/ipc/order";
import { InfoRow, InfoSection } from "./info-section";
import { formatChatTime } from "./format";

export interface CustomerPanelProps {
  selectedConversation: ChannelConversation | null;
  peerName: string;
  buyerFishNick?: string;
  buyerNick?: string;
  accountLabel: string;
  headTitle?: string;
  headPrice?: string;
  headImage?: string;
  itemPrice?: string;
  threadMessages: ChannelMessage[];
  needsReply: boolean;
  lastMessageTime: string;
  buyerOrders: Order[];
  ordersLoading: boolean;
  ordersError: string | null;
}

/** 客户信息栏（右栏）— 买家 / 商品 / 会话概览 / 订单。 */
export function CustomerPanel({
  selectedConversation,
  peerName,
  buyerFishNick,
  buyerNick,
  accountLabel,
  headTitle,
  headPrice,
  headImage,
  itemPrice,
  threadMessages,
  needsReply,
  lastMessageTime,
  buyerOrders,
  ordersLoading,
  ordersError,
}: CustomerPanelProps) {
  return (
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
                <InfoRow label="买家 ID" value={selectedConversation.peer_id || undefined} mono />
                <InfoRow label="鱼塘昵称" value={buyerFishNick} />
                <InfoRow label="买家昵称" value={buyerNick} />
                <InfoRow label="所属账号" value={accountLabel} />
              </InfoSection>

              <InfoSection icon={Package} title="商品信息">
                <InfoRow label="商品" value={headTitle || selectedConversation.item_title || undefined} />
                <InfoRow label="价格" value={headPrice || itemPrice} />
                {headImage ? (
                  <img
                    src={headImage}
                    alt="商品图"
                    className="mt-1 h-24 w-full rounded-lg border border-border object-cover"
                  />
                ) : null}
                <InfoRow label="商品 ID" value={selectedConversation.item_id || undefined} mono />
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
                  <p className="text-[length:var(--text-xs)] text-destructive">{ordersError}</p>
                ) : buyerOrders.length === 0 ? (
                  <p className="text-[length:var(--text-xs)] text-muted-foreground">暂无订单</p>
                ) : (
                  <div className="space-y-2">
                    {buyerOrders.map((order) => (
                      <div key={order.order_no} className="rounded-lg border border-border/70 bg-muted/30 p-2">
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
                            {formatChatTime(order.placed_at)}
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
  );
}

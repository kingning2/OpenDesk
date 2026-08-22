/**
 * 客户会话收件箱 — 会话列表、消息记录、人工发送与客户信息侧栏。
 *
 * 本文件只做编排：数据加载 + 组合三面板（InboxPanel / ThreadPanel / CustomerPanel）。
 */
import { useEffect, useMemo, useState } from "react";
import { Button, PageScaffold, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@desk/ui";
import { RefreshCw } from "@desk/ui/icons";
import { OWNER_ID } from "@desk/platform/constants";
import { accountList, type XianyuAccount } from "@desk/platform/ipc/account";
import {
  channelProductHeadinfo,
  type ProductHeadInfo,
} from "@desk/platform/ipc/channel";
import { orderList, type Order } from "@desk/platform/ipc/order";
import type { ChannelConversation } from "@desk/contracts";
import { formatAmount } from "@desk/utils";
import { managePath } from "@desk/platform/compile";
import { useWorkspaceNav } from "../../app/use-workspace-tabs";
import { conversationNeedsReply, useChannelInbox } from "./use-channel-inbox";
import { InboxPanel, type InboxGroup } from "./inbox-panel";
import { ThreadPanel } from "./thread-panel";
import { CustomerPanel } from "./customer-panel";
import { formatChatTime } from "./format";

/**
 * 客户会话页。
 *
 * @author Xiaoman
 * @created 2026-08-20
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

  useEffect(() => {
    void accountList(OWNER_ID)
      .then(setAccounts)
      .catch(() => setAccounts([]));
  }, []);

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
  const groupedConversations = useMemo<InboxGroup[]>(() => {
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
    ? formatChatTime(threadMessages[threadMessages.length - 1].created_at)
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
        <InboxPanel
          loading={loading}
          conversationsCount={filteredConversations.length}
          groups={groupedConversations}
          messages={messages}
          selectedId={selectedId}
          collapsedAccountIds={collapsedAccountIds}
          onToggleGroup={toggleAccountGroup}
          onSelectConversation={selectConversation}
          onGoToAccounts={() => selectTab(managePath("accounts"))}
        />
        <ThreadPanel
          selectedConversation={selectedConversation}
          peerName={peerName}
          accountLabel={accountLabel}
          threadMessages={threadMessages}
          error={error}
          infoOpen={infoOpen}
          onToggleInfo={() => setInfoOpen((open) => !open)}
          onSend={sendMessage}
        />
        {infoOpen ? (
          <CustomerPanel
            selectedConversation={selectedConversation}
            peerName={peerName}
            buyerFishNick={buyerFishNick}
            buyerNick={buyerNick}
            accountLabel={accountLabel}
            headTitle={headTitle}
            headPrice={headPrice}
            headImage={headImage}
            itemPrice={itemPrice}
            threadMessages={threadMessages}
            needsReply={needsReply}
            lastMessageTime={lastMessageTime}
            buyerOrders={buyerOrders}
            ordersLoading={ordersLoading}
            ordersError={ordersError}
          />
        ) : null}
      </div>
    </PageScaffold>
  );
}

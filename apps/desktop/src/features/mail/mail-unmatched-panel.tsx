/**
 * Unmatched inbound queue — link IMAP messages to customers.
 *
 * @author coisini
 * @created 2026-07-22
 */

import { useCallback, useEffect, useMemo, useState } from "react";
import {
  customerList,
  mailInboxUnmatchedList,
  mailLinkInboundCustomer,
  type CustomerProfile,
  type MailMessage,
} from "@desk/platform";
import { Button, LoadingState, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, cn } from "@desk/ui";
import { useT, useI18n } from "../../i18n";
import { MailDateGroupList } from "./mail-date-group-list";
import { isInboundUnread } from "./mail-read";
import {
  formatMailListTime,
  groupMessagesByDate,
  messageDisplayTime,
} from "./mail-time";

/**
 * List and link unmatched inbound messages.
 *
 * @author coisini
 * @created 2026-07-22
 */
export function MailUnmatchedPanel({
  accountId,
  onLinked,
}: {
  accountId?: string | null;
  onLinked?: () => void;
}) {
  const t = useT();
  const { locale } = useI18n();
  const [messages, setMessages] = useState<MailMessage[]>([]);
  const [customers, setCustomers] = useState<CustomerProfile[]>([]);
  const [loading, setLoading] = useState(true);
  const [linkingId, setLinkingId] = useState("");
  const [selectedCustomerByMessage, setSelectedCustomerByMessage] = useState<Record<string, string>>(
    {},
  );

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const [messageResponse, customerResponse] = await Promise.all([
        mailInboxUnmatchedList({
          account_id: accountId || undefined,
          limit: 50,
          offset: 0,
        }),
        customerList(),
      ]);
      setMessages(messageResponse.items);
      setCustomers(customerResponse.items);
    } finally {
      setLoading(false);
    }
  }, [accountId]);

  useEffect(() => {
    let cancelled = false;
    void Promise.all([
      mailInboxUnmatchedList({
        account_id: accountId || undefined,
        limit: 50,
        offset: 0,
      }),
      customerList(),
    ])
      .then(([messageResponse, customerResponse]) => {
        if (cancelled) {
          return;
        }
        setMessages(messageResponse.items);
        setCustomers(customerResponse.items);
        setLoading(false);
      })
      .catch(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [accountId]);

  const messageGroups = useMemo(
    () =>
      groupMessagesByDate(messages, {
        today: t("mail.list.dateToday"),
        yesterday: t("mail.list.dateYesterday"),
      }, locale),
    [messages, t, locale],
  );

  async function linkMessage(messageId: string) {
    const customerId = selectedCustomerByMessage[messageId];
    if (!customerId) {
      return;
    }
    setLinkingId(messageId);
    try {
      await mailLinkInboundCustomer({ message_id: messageId, customer_id: customerId });
      await load();
      onLinked?.();
    } finally {
      setLinkingId("");
    }
  }

  if (loading) {
    return <LoadingState label={t("mail.sync.loadingUnmatched")} />;
  }

  if (messages.length === 0) {
    return (
      <p className="px-4 py-8 text-center text-xs text-muted-foreground">
        {t("mail.sync.unmatchedEmpty")}
      </p>
    );
  }

  return (
    <MailDateGroupList
      groups={messageGroups}
      renderItem={(message) => (
        <div key={message.id} className="space-y-2 border-b border-border/50 px-3 py-3">
          <div className="flex items-start justify-between gap-2">
            <div className="flex min-w-0 items-center gap-2">
              {isInboundUnread(message) ? (
                <span className="size-2 shrink-0 rounded-full bg-sky-500" aria-hidden />
              ) : null}
              <div
                className={cn(
                  "min-w-0 truncate text-xs",
                  isInboundUnread(message) ? "font-semibold" : "font-medium",
                )}
              >
                {message.subject}
              </div>
            </div>
            <div className="flex shrink-0 flex-col items-end gap-0.5">
              <span className="text-[10px] text-muted-foreground">
                {formatMailListTime(messageDisplayTime(message), locale)}
              </span>
              <span
                className={cn(
                  "text-[9px]",
                  isInboundUnread(message) ? "font-medium text-sky-600" : "text-muted-foreground",
                )}
              >
                {isInboundUnread(message) ? t("mail.list.unread") : t("mail.list.read")}
              </span>
            </div>
          </div>
          <div className="text-[10px] text-muted-foreground">
            {message.from_address ?? t("mail.preview.from")}
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Select
              value={selectedCustomerByMessage[message.id] ?? ""}
              onValueChange={(value) =>
                setSelectedCustomerByMessage((current) => ({ ...current, [message.id]: value }))
              }
            >
              <SelectTrigger className="h-7 min-w-[180px] text-xs">
                <SelectValue placeholder={t("mail.sync.linkCustomer")} />
              </SelectTrigger>
              <SelectContent>
                {customers.map((customer) => (
                  <SelectItem key={customer.id} value={customer.id}>
                    {customer.display_name || customer.email}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button
              size="sm"
              variant="outline"
              disabled={!selectedCustomerByMessage[message.id] || linkingId === message.id}
              onClick={() => void linkMessage(message.id)}
            >
              {linkingId === message.id ? t("mail.sync.linking") : t("mail.sync.linkAction")}
            </Button>
          </div>
        </div>
      )}
    />
  );
}

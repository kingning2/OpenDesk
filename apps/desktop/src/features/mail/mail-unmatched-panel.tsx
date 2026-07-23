/**
 * Unmatched inbound queue — link IMAP messages to customers.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */

import { useCallback, useEffect, useState } from "react";
import {
  customerList,
  mailInboxUnmatchedList,
  mailLinkInboundCustomer,
  type CustomerProfile,
  type MailMessage,
} from "@desk/platform";
import { Button, LoadingState, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@desk/ui";
import { useT } from "../../i18n";

/**
 * List and link unmatched inbound messages.
 *
 * @author Xiaoman
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
    <div className="divide-y divide-border/50">
      {messages.map((message) => (
        <div key={message.id} className="space-y-2 px-3 py-3">
          <div className="text-xs font-medium">{message.subject}</div>
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
      ))}
    </div>
  );
}

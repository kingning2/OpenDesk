/**
 * Mail workbench aligned with email-agent legacy `#tab-mail`:
 * account rail | inbox/sent list | preview + reply | compose modal.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */

import type { ReactNode } from "react";
import { memo, useEffect, useMemo, useRef, useState } from "react";
import { useSearchParams } from "react-router";
import { Mail, Plus, RefreshCw } from "@desk/ui/icons";
import { useT } from "../../i18n";
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  LoadingState,
  PageScaffold,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
  WorkspaceSplit,
  WorkspaceSplitPane,
  WorkspaceSplitRow,
  WorkspaceSplitTitle,
  WorkspaceSplitToolbar,
  cn,
  toast,
} from "@desk/ui";
import { MailHtmlEditor } from "./mail-html-editor";
import { MailHtmlPreview } from "./mail-html-preview";
import { MailUnmatchedPanel } from "./mail-unmatched-panel";
import { stripMailHtml } from "./mail-html";
import { useMailSync } from "./use-mail-sync";
import {
  customerList,
  mailAccountList,
  mailAccountSave,
  mailMessageList,
  mailRecordInbound,
  mailSend,
  mailTemplateApply,
  mailTemplateList,
  mailTemplateSave,
  type CustomerProfile,
  type MailAccount,
  type MailMessage,
  type MailTemplate,
} from "@desk/platform";

/** Stable toast id so StrictMode remounts update one toast instead of stacking two. */
const MAIL_BOOTSTRAP_TOAST_ID = "mail-bootstrap";

type MailboxMode = "inbound" | "outbound";

/**
 * Pick initial mailbox tab from DB counts (no browser storage).
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
function resolveInitialMailbox(
  inboundTotal: number,
  outboundTotal: number,
): MailboxMode {
  if (inboundTotal > 0) {
    return "inbound";
  }
  if (outboundTotal > 0) {
    return "outbound";
  }
  return "inbound";
}

/** Map customer lifecycle to preferred template intent. */
const LIFECYCLE_TEMPLATE_INTENT: Record<string, string> = {
  new: "first_contact",
  contacted: "follow_up",
  negotiating: "quote_proposal",
  won: "cooperation_confirm",
  lost: "follow_up",
  paused: "follow_up",
};

const EMPTY_ACCOUNT_FORM = {
  label: "",
  fromAddress: "",
  fromName: "",
  smtpHost: "",
  smtpPort: "465",
  useTls: true,
  imapHost: "",
  imapPort: "993",
  imapUseTls: true,
  username: "",
  password: "",
};

const EMPTY_TEMPLATE_FORM = {
  name: "",
  templateIntent: "custom",
  subjectTemplate: "",
  bodyTextTemplate: "",
};

const EMPTY_INBOUND_FORM = {
  fromAddress: "",
  subject: "",
  bodyText: "",
};

/**
 * Mail page.
 *
 * @author Xiaoman
 * @created 2026-07-21
 *
 * @returns Mail workbench node
 */
export function MailPage() {
  const t = useT();
  const [searchParams] = useSearchParams();
  const [customers, setCustomers] = useState<CustomerProfile[]>([]);
  const [templates, setTemplates] = useState<MailTemplate[]>([]);
  const [accounts, setAccounts] = useState<MailAccount[]>([]);
  const [messages, setMessages] = useState<MailMessage[]>([]);
  const [messageTotal, setMessageTotal] = useState(0);
  const [mailbox, setMailbox] = useState<MailboxMode>("inbound");
  const [accountFilterId, setAccountFilterId] = useState<string | null>(null);
  const [messageQuery, setMessageQuery] = useState("");
  const [selectedMessageId, setSelectedMessageId] = useState("");
  const [loading, setLoading] = useState(true);
  const [listLoading, setListLoading] = useState(false);
  const [accountModalOpen, setAccountModalOpen] = useState(false);
  const [templateModalOpen, setTemplateModalOpen] = useState(false);
  const [inboundModalOpen, setInboundModalOpen] = useState(false);
  const [composeOpen, setComposeOpen] = useState(false);
  const [composeCustomerId, setComposeCustomerId] = useState("");
  const [composeToAddress, setComposeToAddress] = useState("");
  const [inboundView, setInboundView] = useState<"matched" | "unmatched">("matched");
  const [outboundReplyIndex, setOutboundReplyIndex] = useState<Map<string, MailMessage>>(
    () => new Map(),
  );
  const lastSeenSyncAtRef = useRef<string>("");

  const mailSync = useMailSync(accountFilterId);

  const selectedMessage = useMemo(
    () => messages.find((item) => item.id === selectedMessageId) ?? null,
    [messages, selectedMessageId],
  );

  const customerById = useMemo(() => {
    const map = new Map<string, CustomerProfile>();
    for (const item of customers) {
      map.set(item.id, item);
    }
    return map;
  }, [customers]);

  async function fetchBootstrap() {
    const [customerResponse, templateResponse, accountResponse] = await Promise.all([
      customerList(),
      mailTemplateList(),
      mailAccountList(),
    ]);
    return {
      customers: customerResponse.items,
      templates: templateResponse.items,
      accounts: accountResponse.items,
    };
  }

  async function fetchMessages(options?: {
    mailbox?: MailboxMode;
    accountId?: string | null;
    query?: string;
  }) {
    const direction = options?.mailbox ?? mailbox;
    const accountId = options?.accountId === undefined ? accountFilterId : options.accountId;
    const query = options?.query === undefined ? messageQuery : options.query;
    const response = await mailMessageList({
      direction,
      account_id: accountId || undefined,
      query: query.trim() || undefined,
      limit: 100,
      offset: 0,
    });
    return response;
  }

  async function refreshOutboundReplyIndex(accountId?: string | null) {
    const response = await mailMessageList({
      direction: "outbound",
      account_id: accountId || undefined,
      limit: 200,
      offset: 0,
    });
    setOutboundReplyIndex(buildOutboundReplyIndex(response.items));
  }

  async function refreshMessages(options?: {
    mailbox?: MailboxMode;
    accountId?: string | null;
    query?: string;
    keepSelection?: boolean;
    selectMessageId?: string;
  }) {
    const direction = options?.mailbox ?? mailbox;
    const accountId = options?.accountId === undefined ? accountFilterId : options.accountId;
    setListLoading(true);
    try {
      const response = await fetchMessages(options);
      setMessages(response.items);
      setMessageTotal(response.total);
      if (options?.selectMessageId) {
        setSelectedMessageId(
          response.items.some((item) => item.id === options.selectMessageId)
            ? options.selectMessageId
            : (response.items[0]?.id ?? ""),
        );
      } else if (!options?.keepSelection) {
        setSelectedMessageId(response.items[0]?.id ?? "");
      } else if (!response.items.some((item) => item.id === selectedMessageId)) {
        setSelectedMessageId(response.items[0]?.id ?? "");
      }
      if (direction === "inbound") {
        void refreshOutboundReplyIndex(accountId);
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setListLoading(false);
    }
  }

  async function afterOutboundSent(messageId?: string) {
    setMailbox("outbound");
    await refreshMessages({ mailbox: "outbound", selectMessageId: messageId });
  }

  useEffect(() => {
    const signal = { cancelled: false };
    toast.loading(t("mail.statusLoading"), { id: MAIL_BOOTSTRAP_TOAST_ID });
    void fetchBootstrap()
      .then(async (snapshot) => {
        if (signal.cancelled) {
          return;
        }
        setCustomers(snapshot.customers);
        setTemplates(snapshot.templates);
        setAccounts(snapshot.accounts);

        const queryCustomerId = searchParams.get("customerId")?.trim() || "";
        if (queryCustomerId && snapshot.customers.some((item) => item.id === queryCustomerId)) {
          const matched = snapshot.customers.find((item) => item.id === queryCustomerId);
          setComposeCustomerId(queryCustomerId);
          setComposeToAddress(matched?.email || "");
          setComposeOpen(true);
        }

        const [inboundResponse, outboundResponse] = await Promise.all([
          fetchMessages({ mailbox: "inbound", accountId: null, query: "" }),
          fetchMessages({ mailbox: "outbound", accountId: null, query: "" }),
        ]);
        const initialMailbox = resolveInitialMailbox(
          inboundResponse.total,
          outboundResponse.total,
        );
        const messageResponse = initialMailbox === "outbound" ? outboundResponse : inboundResponse;
        if (signal.cancelled) {
          return;
        }
        setMailbox(initialMailbox);
        setMessages(messageResponse.items);
        setMessageTotal(messageResponse.total);
        setSelectedMessageId(messageResponse.items[0]?.id ?? "");
        setOutboundReplyIndex(buildOutboundReplyIndex(outboundResponse.items));
        toast.success(t("mail.statusReady"), { id: MAIL_BOOTSTRAP_TOAST_ID });
      })
      .catch((error: unknown) => {
        if (signal.cancelled) {
          return;
        }
        toast.error(error instanceof Error ? error.message : String(error), {
          id: MAIL_BOOTSTRAP_TOAST_ID,
        });
      })
      .finally(() => {
        if (!signal.cancelled) {
          setLoading(false);
        }
      });
    return () => {
      signal.cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [searchParams]);

  function switchMailbox(mode: MailboxMode) {
    setMailbox(mode);
    void refreshMessages({ mailbox: mode });
  }

  function selectAccountFilter(id: string | null) {
    setAccountFilterId(id);
    void refreshMessages({ accountId: id });
  }

  useEffect(() => {
    if (loading) {
      return;
    }
    const timer = window.setTimeout(() => {
      void refreshMessages({ query: messageQuery, keepSelection: true });
    }, 400);
    return () => {
      window.clearTimeout(timer);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [messageQuery]);

  useEffect(() => {
    if (loading || mailbox !== "inbound" || !mailSync.lastSyncAt) {
      return;
    }
    if (!lastSeenSyncAtRef.current) {
      lastSeenSyncAtRef.current = mailSync.lastSyncAt;
      return;
    }
    if (lastSeenSyncAtRef.current === mailSync.lastSyncAt) {
      return;
    }
    lastSeenSyncAtRef.current = mailSync.lastSyncAt;
    void refreshMessages({ keepSelection: true });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [loading, mailbox, mailSync.lastSyncAt]);

  function openCompose(options?: { customerId?: string; toAddress?: string }) {
    const preferredCustomer =
      options?.customerId ||
      selectedMessage?.customer_id ||
      "";
    setComposeCustomerId(preferredCustomer);
    setComposeToAddress(
      options?.toAddress ||
        selectedMessage?.to_address ||
        customers.find((item) => item.id === preferredCustomer)?.email ||
        "",
    );
    setComposeOpen(true);
  }

  return (
    <PageScaffold fill containerPadding="none">
      <WorkspaceSplit className="min-h-0 flex-1 border-0" defaultStartWidth={168} minStartWidth={140} maxStartWidth={220}>
        <WorkspaceSplitPane
          side="start"
          header={
            <WorkspaceSplitToolbar className="justify-between px-3 py-2">
              <WorkspaceSplitTitle className="text-xs">{t("mail.accounts.title")}</WorkspaceSplitTitle>
              <Button
                size="sm"
                variant="ghost"
                className="size-7 p-0"
                disabled={loading || listLoading}
                onClick={() => void refreshMessages({ keepSelection: true })}
              >
                <RefreshCw className={cn("size-3.5", (loading || listLoading) && "animate-spin")} />
              </Button>
            </WorkspaceSplitToolbar>
          }
        >
          <div className="flex flex-col">
            <WorkspaceSplitRow
              active={accountFilterId === null}
              className="px-3 py-2 text-xs"
              onClick={() => selectAccountFilter(null)}
            >
              <span className="font-medium">{t("mail.accounts.all")}</span>
              <span className="mt-0.5 block text-[10px] text-muted-foreground">{accounts.length}</span>
            </WorkspaceSplitRow>
            {accounts.map((account) => (
              <WorkspaceSplitRow
                key={account.id}
                active={accountFilterId === account.id}
                className="px-3 py-2 text-xs"
                onClick={() => selectAccountFilter(account.id)}
              >
                <span className="inline-flex items-center gap-1.5 font-medium">
                  <Mail className="size-3 shrink-0 text-sky-500" />
                  <span className="truncate">{account.label}</span>
                </span>
                <span className="mt-0.5 block truncate text-[10px] text-muted-foreground">
                  {account.from_address}
                </span>
              </WorkspaceSplitRow>
            ))}
            <button
              type="button"
              className="flex items-center gap-1.5 border-b border-border/50 px-3 py-2.5 text-left text-xs text-primary hover:bg-muted/40"
              onClick={() => setAccountModalOpen(true)}
            >
              <Plus className="size-3.5" />
              {t("mail.accounts.bind")}
            </button>
          </div>
        </WorkspaceSplitPane>

        <WorkspaceSplitPane scroll={false}>
          <div className="flex h-full min-h-0 flex-col">
            <div className="flex flex-wrap items-center gap-2 border-b border-border/50 px-3 py-2">
              <Button
                size="sm"
                variant={mailbox === "inbound" ? "default" : "outline"}
                onClick={() => switchMailbox("inbound")}
              >
                {t("mail.mailbox.inbox")}
              </Button>
              <Button
                size="sm"
                variant={mailbox === "outbound" ? "default" : "outline"}
                onClick={() => switchMailbox("outbound")}
              >
                {t("mail.mailbox.sent")}
              </Button>
              <Button size="sm" onClick={() => openCompose()}>
                {t("mail.compose.open")}
              </Button>
              <Button
                size="sm"
                variant="outline"
                disabled={mailSync.isSyncing || loading}
                onClick={() => {
                  void mailSync.syncNow().then(() => {
                    toast.success(t("mail.sync.enqueued"));
                    void refreshMessages({ keepSelection: true });
                  });
                }}
              >
                {mailSync.isSyncing ? t("mail.sync.syncing") : t("mail.sync.now")}
              </Button>
              <Button size="sm" variant="outline" onClick={() => setInboundModalOpen(true)}>
                {t("mail.inbound.record")}
              </Button>
              <Button size="sm" variant="ghost" onClick={() => setTemplateModalOpen(true)}>
                {t("mail.compose.manageTemplates")}
              </Button>
              {mailbox === "inbound" ? (
                <>
                  <Button
                    size="sm"
                    variant={inboundView === "matched" ? "default" : "outline"}
                    onClick={() => setInboundView("matched")}
                  >
                    {t("mail.sync.matched")}
                  </Button>
                  <Button
                    size="sm"
                    variant={inboundView === "unmatched" ? "default" : "outline"}
                    onClick={() => setInboundView("unmatched")}
                  >
                    {t("mail.sync.unmatched")}
                  </Button>
                </>
              ) : null}
              {mailSync.lastError ? (
                <span className="text-[10px] text-destructive">
                  {t("mail.sync.lastError", { error: mailSync.lastError })}
                </span>
              ) : mailSync.lastSyncAt ? (
                <span className="text-[10px] text-muted-foreground">
                  {t("mail.sync.lastSync", { time: mailSync.lastSyncAt })}
                </span>
              ) : null}
              <span className="ml-auto text-[10px] text-muted-foreground">
                {t("mail.list.total", { count: messageTotal })}
              </span>
            </div>

            <div className="border-b border-border/50 px-3 py-2">
              <Input
                value={messageQuery}
                onChange={(event) => setMessageQuery(event.target.value)}
                placeholder={t("mail.list.searchPlaceholder")}
                className="h-7 text-xs"
              />
            </div>

            <WorkspaceSplit
              className="min-h-0 flex-1 border-0"
              defaultStartWidth={360}
              minStartWidth={280}
              maxStartWidth={520}
            >
              <WorkspaceSplitPane side="start">
                {mailbox === "inbound" && inboundView === "unmatched" ? (
                  <MailUnmatchedPanel
                    accountId={accountFilterId}
                    onLinked={() => void refreshMessages({ keepSelection: true })}
                  />
                ) : loading || listLoading ? (
                  <LoadingState label={t("mail.statusLoading")} />
                ) : messages.length === 0 ? (
                  <div className="space-y-2 px-4 py-10 text-center">
                    <p className="text-xs text-muted-foreground">{t("mail.list.empty")}</p>
                    <p className="text-[10px] text-muted-foreground">{t("mail.list.emptyHint")}</p>
                  </div>
                ) : (
                  messages.map((item) => {
                    const customer = item.customer_id
                      ? customerById.get(item.customer_id)
                      : undefined;
                    return (
                      <WorkspaceSplitRow
                        key={item.id}
                        active={selectedMessageId === item.id}
                        className="px-3 py-2.5"
                        onClick={() => setSelectedMessageId(item.id)}
                      >
                        <div className="truncate text-sm font-medium">
                          {messageCounterpartyLabel(item, customer)}
                        </div>
                        <div className="mt-0.5 truncate text-xs text-muted-foreground">
                          {messageAddressSubline(item)}
                        </div>
                        <div className="mt-1 flex items-center gap-2 text-[10px] text-muted-foreground">
                          <span
                            className={cn(
                              item.status === "failed" && "text-destructive",
                              item.status === "sent" && "text-emerald-600",
                            )}
                          >
                            {item.status}
                          </span>
                          {item.open_tracking_id ? (
                            <span className="text-sky-600">{t("mail.preview.openTracking")}</span>
                          ) : null}
                          <span>{formatMailTime(item.sent_at || item.received_at || item.created_at)}</span>
                        </div>
                      </WorkspaceSplitRow>
                    );
                  })
                )}
              </WorkspaceSplitPane>

              <WorkspaceSplitPane
                scroll={false}
                header={
                  <WorkspaceSplitToolbar className="justify-between px-4 py-2">
                    <div className="min-w-0">
                      <WorkspaceSplitTitle className="truncate text-sm">
                        {selectedMessage?.subject || t("mail.preview.emptyTitle")}
                      </WorkspaceSplitTitle>
                      {selectedMessage?.customer_id ? (
                        <p className="truncate text-xs text-muted-foreground">
                          {customerById.get(selectedMessage.customer_id)?.email ||
                            selectedMessage.customer_id}
                        </p>
                      ) : null}
                    </div>
                    {selectedMessage ? (
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() =>
                          openCompose({
                            customerId: selectedMessage.customer_id,
                            toAddress: selectedMessage.to_address,
                          })
                        }
                      >
                        {selectedMessage.direction === "outbound"
                          ? t("mail.preview.followUp")
                          : t("mail.compose.open")}
                      </Button>
                    ) : null}
                  </WorkspaceSplitToolbar>
                }
              >
                {!selectedMessage ? (
                  <div className="flex h-full items-center justify-center px-6 text-center text-sm text-muted-foreground">
                    {t("mail.preview.empty")}
                  </div>
                ) : (
                  <PreviewReplyPane
                    key={selectedMessage.id}
                    message={selectedMessage}
                    repliedOutbound={resolveRepliedOutbound(selectedMessage, outboundReplyIndex)}
                    templates={templates}
                    accounts={accounts}
                    customers={customers}
                    onComposeFollowUp={() =>
                      openCompose({
                        customerId: selectedMessage.customer_id,
                        toAddress: selectedMessage.to_address,
                      })
                    }
                    onSent={(messageId) => void afterOutboundSent(messageId)}
                  />
                )}
              </WorkspaceSplitPane>
            </WorkspaceSplit>
          </div>
        </WorkspaceSplitPane>
      </WorkspaceSplit>

      {composeOpen ? (
        <ComposeModal
          customers={customers}
          templates={templates}
          accounts={accounts}
          initialCustomerId={composeCustomerId}
          initialToAddress={composeToAddress}
          onClose={() => setComposeOpen(false)}
          onSent={async (messageId) => {
            setComposeOpen(false);
            await afterOutboundSent(messageId);
          }}
        />
      ) : null}

      {accountModalOpen ? (
        <AccountBindModal
          onClose={() => setAccountModalOpen(false)}
          onSaved={(items) => {
            setAccounts(items);
            setAccountModalOpen(false);
          }}
        />
      ) : null}

      {templateModalOpen ? (
        <TemplateSaveModal
          onClose={() => setTemplateModalOpen(false)}
          onSaved={(items) => {
            setTemplates(items);
            setTemplateModalOpen(false);
          }}
        />
      ) : null}

      {inboundModalOpen ? (
        <InboundRecordModal
          customers={customers}
          initialCustomerId={selectedMessage?.customer_id || customers[0]?.id || ""}
          onClose={() => setInboundModalOpen(false)}
          onSaved={async () => {
            setInboundModalOpen(false);
            setMailbox("inbound");
            await refreshMessages({ mailbox: "inbound" });
          }}
        />
      ) : null}
    </PageScaffold>
  );
}

/**
 * Format mail timestamp millis / ISO-ish strings for list rows.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
function formatMailTime(raw: string): string {
  const asNumber = Number(raw);
  if (Number.isFinite(asNumber) && asNumber > 0) {
    return new Date(asNumber).toLocaleString();
  }
  const parsed = Date.parse(raw);
  if (Number.isFinite(parsed)) {
    return new Date(parsed).toLocaleString();
  }
  return raw;
}

const CUSTOMER_NONE = "__none__";

/**
 * Resolve outbound body fields (plain text required by contract; HTML optional).
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
function resolveOutboundBody(
  bodyText: string,
  bodyHtml: string,
): { body_text: string; body_html?: string } | null {
  const html = bodyHtml.trim();
  const text = bodyText.trim() || (html ? stripMailHtml(html) : "");
  if (!text) {
    return null;
  }
  return {
    body_text: text,
    body_html: html || undefined,
  };
}

/**
 * Extract bare email from a RFC-like address string.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
function extractEmailAddress(value: string | undefined): string {
  if (!value) {
    return "";
  }
  const match = value.match(/<([^>]+)>/);
  if (match?.[1]) {
    return match[1].trim();
  }
  return value.trim();
}

/**
 * Normalize RFC Message-ID for matching reply chains.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
function normalizeMessageId(value: string): string {
  return value.trim().replace(/^<|>$/g, "");
}

/**
 * Build quick lookup from outbound RFC Message-ID to message row.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
function buildOutboundReplyIndex(items: MailMessage[]): Map<string, MailMessage> {
  const map = new Map<string, MailMessage>();
  for (const item of items) {
    const raw = item.rfc_message_id?.trim();
    if (!raw) {
      continue;
    }
    map.set(raw, item);
    const normalized = normalizeMessageId(raw);
    if (normalized && normalized !== raw) {
      map.set(normalized, item);
    }
  }
  return map;
}

/**
 * Resolve which outbound message an inbound reply references.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
function resolveRepliedOutbound(
  message: MailMessage | null,
  index: Map<string, MailMessage>,
): MailMessage | null {
  if (!message || message.direction !== "inbound") {
    return null;
  }
  const candidates: string[] = [];
  if (message.in_reply_to) {
    candidates.push(message.in_reply_to.trim());
    candidates.push(normalizeMessageId(message.in_reply_to));
  }
  if (message.references) {
    for (const token of message.references.split(/\s+/)) {
      const trimmed = token.trim();
      if (!trimmed) {
        continue;
      }
      candidates.push(trimmed);
      candidates.push(normalizeMessageId(trimmed));
    }
  }
  for (const candidate of candidates) {
    const found = index.get(candidate);
    if (found) {
      return found;
    }
  }
  return null;
}

/**
 * Resolve list/preview counterparty label (inbound from / outbound to).
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
function messageCounterpartyLabel(
  message: MailMessage,
  customer: CustomerProfile | undefined,
): string {
  if (customer?.display_name || customer?.email) {
    return customer.display_name || customer.email || message.subject;
  }
  if (message.direction === "inbound") {
    return message.from_address || message.subject;
  }
  return message.to_address || message.subject;
}

/**
 * Resolve address subline for list rows.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
function messageAddressSubline(message: MailMessage): string {
  const address =
    message.direction === "inbound" ? message.from_address : message.to_address;
  if (!address) {
    return message.subject;
  }
  return `${address} · ${message.subject}`;
}

/**
 * Basic email shape check for compose/reply.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
function isValidEmail(value: string): boolean {
  const trimmed = value.trim();
  return trimmed.includes("@") && trimmed.length > 3 && !trimmed.includes(" ");
}

/**
 * Resolve default reply recipient from message / linked customer.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
function resolveReplyToAddress(
  message: MailMessage,
  customer: CustomerProfile | undefined,
): string {
  if (customer?.email) {
    return customer.email;
  }
  if (message.direction === "inbound") {
    const fromEmail = extractEmailAddress(message.from_address);
    if (fromEmail) {
      return fromEmail;
    }
  }
  if (message.to_address) {
    return message.to_address;
  }
  return "";
}

/**
 * Right pane: message preview + reply composer (legacy `pv-reply` pattern).
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
const PreviewReplyPane = memo(function PreviewReplyPane({
  message,
  repliedOutbound,
  templates,
  accounts,
  customers,
  onComposeFollowUp,
  onSent,
}: {
  message: MailMessage;
  repliedOutbound: MailMessage | null;
  templates: MailTemplate[];
  accounts: MailAccount[];
  customers: CustomerProfile[];
  onComposeFollowUp: () => void;
  onSent: (messageId?: string) => void;
}) {
  const t = useT();
  const customer = customers.find((item) => item.id === message.customer_id);
  const preferredIntent = customer
    ? (LIFECYCLE_TEMPLATE_INTENT[customer.lifecycle_status] ?? "follow_up")
    : "follow_up";
  const [templateId, setTemplateId] = useState(
    () => templates.find((item) => item.template_intent === preferredIntent)?.id || templates[0]?.id || "",
  );
  const [accountId, setAccountId] = useState(() => message.account_id || accounts[0]?.id || "");
  const [toAddress, setToAddress] = useState(() => resolveReplyToAddress(message, customer));
  const [subject, setSubject] = useState("");
  const [bodyText, setBodyText] = useState("");
  const [bodyHtml, setBodyHtml] = useState("");
  const [sending, setSending] = useState(false);
  const [sendConfirmOpen, setSendConfirmOpen] = useState(false);
  const [openTrackingEnabled, setOpenTrackingEnabled] = useState(true);
  const isInbound = message.direction === "inbound";
  const quickTemplates = templates.slice(0, 6);

  async function applyTemplate(nextTemplateId = templateId) {
    if (!message.customer_id || !nextTemplateId) {
      toast.error(t("mail.statusSelectFirst"));
      return;
    }
    try {
      const response = await mailTemplateApply({
        customer_id: message.customer_id,
        template_id: nextTemplateId,
        account_id: accountId || undefined,
      });
      setTemplateId(nextTemplateId);
      setSubject(response.subject);
      setBodyText(response.body_text);
      setBodyHtml(response.body_html ?? "");
      toast.success(t("mail.statusTemplateApplied"));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  function requestSendReply() {
    const recipient = toAddress.trim();
    if (!isValidEmail(recipient)) {
      toast.error(t("mail.statusToAddressInvalid"));
      return;
    }
    if (!accountId || !subject.trim() || !resolveOutboundBody(bodyText, bodyHtml)) {
      toast.error(t("mail.statusSendValidation"));
      return;
    }
    setSendConfirmOpen(true);
  }

  async function confirmSendReply() {
    const recipient = toAddress.trim();
    const body = resolveOutboundBody(bodyText, bodyHtml);
    if (!body) {
      toast.error(t("mail.statusSendValidation"));
      return;
    }
    setSendConfirmOpen(false);
    setSending(true);
    try {
      const response = await mailSend({
        to_address: recipient,
        customer_id: message.customer_id || undefined,
        account_id: accountId,
        template_id: templateId || undefined,
        subject,
        body_text: body.body_text,
        body_html: body.body_html,
        open_tracking_enabled: openTrackingEnabled,
      });
      if (response.status === "sent") {
        toast.success(t("mail.statusSent", { id: response.message_id }));
      } else {
        toast.error(t("mail.statusSendFailed", { id: response.message_id }));
      }
      onSent(response.message_id);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSending(false);
    }
  }

  return (
    <>
      <ConfirmDialog
        open={sendConfirmOpen}
        title={t("mail.compose.confirmSendTitle")}
        description={t("mail.compose.confirmSend")}
        confirmLabel={t("mail.compose.confirmSendAction")}
        cancelLabel={t("mail.modal.cancel")}
        loading={sending}
        onConfirm={() => void confirmSendReply()}
        onClose={() => setSendConfirmOpen(false)}
      />
    <div className="flex h-full min-h-0 flex-col">
      <div className="min-h-[180px] flex-1 overflow-y-auto p-4">
        <div className="rounded-[var(--radius-md)] border border-border/60 bg-muted/20 p-3">
          <p className="mb-1 text-xs text-muted-foreground">
            {isInbound ? t("mail.preview.metaInbound") : t("mail.preview.metaOutbound")} ·{" "}
            {message.status}
            {message.error_message ? ` · ${message.error_message}` : ""}
          </p>
          {message.from_address || message.to_address ? (
            <p className="mb-2 text-xs text-muted-foreground">
              {message.from_address ? (
                <span>
                  {t("mail.preview.from")}: {message.from_address}
                </span>
              ) : null}
              {message.from_address && message.to_address ? " · " : null}
              {message.to_address ? (
                <span>
                  {t("mail.preview.to")}: {message.to_address}
                </span>
              ) : null}
              {message.open_tracking_id ? (
                <span className="ml-1 text-sky-600">· {t("mail.preview.openTracking")}</span>
              ) : null}
            </p>
          ) : null}
          {isInbound ? (
            <p className="mb-2 text-xs text-muted-foreground">
              {repliedOutbound
                ? t("mail.preview.replyToKnown", {
                    subject: repliedOutbound.subject || repliedOutbound.id,
                  })
                : t("mail.preview.replyToUnknown")}
            </p>
          ) : null}
          <h3 className="mb-3 text-sm font-semibold leading-snug">{message.subject || t("mail.preview.emptyTitle")}</h3>
          <MailHtmlPreview
            bodyHtml={message.body_html}
            bodyText={message.body_text}
            emptyLabel={t("mail.preview.emptyBody")}
            minHeight={200}
          />
        </div>
      </div>

      {isInbound ? (
        <div className="max-h-[48%] shrink-0 space-y-2 overflow-y-auto border-t border-border/50 p-4">
          {quickTemplates.length > 0 ? (
            <div className="space-y-1.5">
              <p className="text-[10px] text-muted-foreground">{t("mail.compose.quickReplies")}</p>
              <div className="flex flex-wrap gap-1.5">
                {quickTemplates.map((item) => (
                  <button
                    key={item.id}
                    type="button"
                    className={cn(
                      "rounded-md border px-2 py-1 text-[11px] transition-colors",
                      templateId === item.id
                        ? "border-primary bg-primary/10 text-primary"
                        : "border-border/70 text-muted-foreground hover:bg-muted/50",
                    )}
                    onClick={() => void applyTemplate(item.id)}
                  >
                    {item.name}
                  </button>
                ))}
              </div>
            </div>
          ) : null}

          <div className="grid gap-2 md:grid-cols-2">
            <Field label={t("mail.compose.toAddress")} className="md:col-span-2">
              <Input
                value={toAddress}
                onChange={(event) => setToAddress(event.target.value)}
                placeholder={t("mail.compose.toAddressPlaceholder")}
              />
            </Field>
            <Field label={t("mail.compose.template")}>
              <Select value={templateId} onValueChange={setTemplateId}>
                <SelectTrigger>
                  <SelectValue placeholder={t("mail.compose.templatePlaceholder")} />
                </SelectTrigger>
                <SelectContent>
                  {templates.map((item) => (
                    <SelectItem key={item.id} value={item.id}>
                      {item.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </Field>
            <Field label={t("mail.compose.account")}>
              <Select value={accountId} onValueChange={setAccountId}>
                <SelectTrigger>
                  <SelectValue placeholder={t("mail.compose.accountPlaceholder")} />
                </SelectTrigger>
                <SelectContent>
                  {accounts.map((item) => (
                    <SelectItem key={item.id} value={item.id}>
                      {item.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </Field>
          </div>
          <Button size="sm" variant="outline" className="w-fit" onClick={() => void applyTemplate()}>
            {t("mail.compose.applyTemplate")}
          </Button>
          <Field label={t("mail.compose.subject")}>
            <Input value={subject} onChange={(event) => setSubject(event.target.value)} />
          </Field>
          <Field label={t("mail.compose.reply")} className="flex flex-col">
            <MailHtmlEditor
              bodyText={bodyText}
              bodyHtml={bodyHtml}
              onBodyTextChange={setBodyText}
              onBodyHtmlChange={setBodyHtml}
              textLabel={t("mail.compose.bodyModeText")}
              htmlLabel={t("mail.compose.bodyModeHtml")}
              previewLabel={t("mail.compose.htmlPreview")}
              textPlaceholder={t("mail.compose.replyPlaceholder")}
              htmlPlaceholder={t("mail.compose.htmlPlaceholder")}
              emptyPreviewLabel={t("mail.preview.emptyBody")}
            />
          </Field>
          <OpenTrackingSwitch
            enabled={openTrackingEnabled}
            onEnabledChange={setOpenTrackingEnabled}
            disabled={sending}
          />
          <Button size="sm" onClick={() => void requestSendReply()} disabled={sending}>
            {sending ? t("mail.compose.sending") : t("mail.compose.send")}
          </Button>
        </div>
      ) : (
        <div className="flex shrink-0 items-center justify-between border-t border-border/50 px-4 py-3">
          <p className="text-xs text-muted-foreground">{t("mail.preview.metaOutbound")}</p>
          <Button size="sm" variant="outline" onClick={onComposeFollowUp}>
            {t("mail.preview.followUp")}
          </Button>
        </div>
      )}
    </div>
    </>
  );
});


/**
 * Toggle outbound open-tracking pixel injection.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
function OpenTrackingSwitch({
  enabled,
  onEnabledChange,
  disabled = false,
}: {
  enabled: boolean;
  onEnabledChange: (value: boolean) => void;
  disabled?: boolean;
}) {
  const t = useT();
  return (
    <div className="flex items-center justify-between gap-3 rounded-[var(--radius-md)] border border-border/60 bg-muted/20 px-3 py-2">
      <div className="min-w-0">
        <p className="text-xs font-medium text-foreground">{t("mail.compose.openTracking")}</p>
        <p className="text-[10px] leading-snug text-muted-foreground">
          {t("mail.compose.openTrackingHint")}
        </p>
      </div>
      <Switch
        checked={enabled}
        onCheckedChange={onEnabledChange}
        disabled={disabled}
        aria-label={t("mail.compose.openTracking")}
      />
    </div>
  );
}

/**
 * In-app confirm dialog (replaces browser `window.confirm`).
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
function ConfirmDialog({
  open,
  title,
  description,
  confirmLabel,
  cancelLabel,
  loading = false,
  onConfirm,
  onClose,
}: {
  open: boolean;
  title: string;
  description: string;
  confirmLabel: string;
  cancelLabel: string;
  loading?: boolean;
  onConfirm: () => void;
  onClose: () => void;
}) {
  return (
    <Dialog
      open={open}
      onOpenChange={(nextOpen) => {
        if (!nextOpen && !loading) {
          onClose();
        }
      }}
    >
      <DialogContent className="max-w-sm">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={loading}>
            {cancelLabel}
          </Button>
          <Button onClick={onConfirm} disabled={loading}>
            {confirmLabel}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

/**
 * Compose modal — legacy `openCompose()` flow with OpenDesk template-first send.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
const ComposeModal = memo(function ComposeModal({
  customers,
  templates,
  accounts,
  initialCustomerId,
  initialToAddress,
  onClose,
  onSent,
}: {
  customers: CustomerProfile[];
  templates: MailTemplate[];
  accounts: MailAccount[];
  initialCustomerId: string;
  initialToAddress?: string;
  onClose: () => void;
  onSent: (messageId?: string) => void | Promise<void>;
}) {
  const t = useT();
  const initialCustomer =
    customers.find((item) => item.id === initialCustomerId) ?? null;
  const [customerId, setCustomerId] = useState(initialCustomer?.id || CUSTOMER_NONE);
  const selectedCustomer =
    customerId === CUSTOMER_NONE ? undefined : customers.find((item) => item.id === customerId);
  const preferredIntent = selectedCustomer
    ? (LIFECYCLE_TEMPLATE_INTENT[selectedCustomer.lifecycle_status] ?? "first_contact")
    : "first_contact";
  const [toAddress, setToAddress] = useState(
    initialToAddress || initialCustomer?.email || "",
  );
  const [templateId, setTemplateId] = useState(
    () => templates.find((item) => item.template_intent === preferredIntent)?.id || templates[0]?.id || "",
  );
  const [accountId, setAccountId] = useState(accounts[0]?.id || "");
  const [subject, setSubject] = useState("");
  const [bodyText, setBodyText] = useState("");
  const [bodyHtml, setBodyHtml] = useState("");
  const [sending, setSending] = useState(false);
  const [sendConfirmOpen, setSendConfirmOpen] = useState(false);
  const [openTrackingEnabled, setOpenTrackingEnabled] = useState(true);

  async function applyTemplate() {
    if (!selectedCustomer || !templateId) {
      toast.error(t("mail.statusSelectFirst"));
      return;
    }
    try {
      const response = await mailTemplateApply({
        customer_id: selectedCustomer.id,
        template_id: templateId,
        account_id: accountId || undefined,
      });
      setSubject(response.subject);
      setBodyText(response.body_text);
      setBodyHtml(response.body_html ?? "");
      if (!toAddress.trim()) {
        setToAddress(selectedCustomer.email);
      }
      toast.success(t("mail.statusTemplateApplied"));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  function requestSendOutbound() {
    const recipient = toAddress.trim();
    if (!isValidEmail(recipient)) {
      toast.error(t("mail.statusToAddressInvalid"));
      return;
    }
    if (!accountId || !subject.trim() || !resolveOutboundBody(bodyText, bodyHtml)) {
      toast.error(t("mail.statusSendValidation"));
      return;
    }
    setSendConfirmOpen(true);
  }

  async function confirmSendOutbound() {
    const recipient = toAddress.trim();
    const body = resolveOutboundBody(bodyText, bodyHtml);
    if (!body) {
      toast.error(t("mail.statusSendValidation"));
      return;
    }
    setSendConfirmOpen(false);
    setSending(true);
    try {
      const response = await mailSend({
        to_address: recipient,
        customer_id: selectedCustomer?.id,
        account_id: accountId,
        template_id: templateId || undefined,
        subject,
        body_text: body.body_text,
        body_html: body.body_html,
        open_tracking_enabled: openTrackingEnabled,
      });
      if (response.status === "sent") {
        toast.success(t("mail.statusSent", { id: response.message_id }));
      } else {
        toast.error(t("mail.statusSendFailed", { id: response.message_id }));
      }
      await onSent(response.message_id);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSending(false);
    }
  }

  return (
    <>
      <ConfirmDialog
        open={sendConfirmOpen}
        title={t("mail.compose.confirmSendTitle")}
        description={t("mail.compose.confirmSend")}
        confirmLabel={t("mail.compose.confirmSendAction")}
        cancelLabel={t("mail.modal.cancel")}
        loading={sending}
        onConfirm={() => void confirmSendOutbound()}
        onClose={() => setSendConfirmOpen(false)}
      />
    <Modal
      title={t("mail.compose.title")}
      onClose={onClose}
      footer={
        <>
          <Button size="sm" variant="outline" onClick={onClose}>
            {t("mail.modal.cancel")}
          </Button>
          <Button size="sm" variant="outline" onClick={() => void applyTemplate()}>
            {t("mail.compose.applyTemplate")}
          </Button>
          <Button size="sm" onClick={() => void requestSendOutbound()} disabled={sending}>
            {sending ? t("mail.compose.sending") : t("mail.compose.send")}
          </Button>
        </>
      }
    >
      <div className="grid gap-3 md:grid-cols-2">
        <Field label={t("mail.compose.toAddress")} className="md:col-span-2">
          <Input
            value={toAddress}
            onChange={(event) => setToAddress(event.target.value)}
            placeholder={t("mail.compose.toAddressPlaceholder")}
          />
        </Field>
        <Field label={t("mail.compose.customer")}>
          <Select
            value={customerId || CUSTOMER_NONE}
            onValueChange={(value) => {
              setCustomerId(value);
              if (value === CUSTOMER_NONE) {
                return;
              }
              const next = customers.find((item) => item.id === value);
              if (next) {
                setToAddress(next.email);
                const intent = LIFECYCLE_TEMPLATE_INTENT[next.lifecycle_status] ?? "first_contact";
                const preferred = templates.find((item) => item.template_intent === intent);
                if (preferred) {
                  setTemplateId(preferred.id);
                }
              }
            }}
          >
            <SelectTrigger>
              <SelectValue placeholder={t("mail.compose.customerPlaceholder")} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={CUSTOMER_NONE}>{t("mail.compose.customerNone")}</SelectItem>
              {customers.map((item) => (
                <SelectItem key={item.id} value={item.id}>
                  {item.display_name || item.email}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
        <Field label={t("mail.compose.account")}>
          <Select value={accountId} onValueChange={setAccountId}>
            <SelectTrigger>
              <SelectValue placeholder={t("mail.compose.accountPlaceholder")} />
            </SelectTrigger>
            <SelectContent>
              {accounts.map((item) => (
                <SelectItem key={item.id} value={item.id}>
                  {item.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
        <Field label={t("mail.compose.template")} className="md:col-span-2">
          <Select value={templateId} onValueChange={setTemplateId}>
            <SelectTrigger>
              <SelectValue placeholder={t("mail.compose.templatePlaceholder")} />
            </SelectTrigger>
            <SelectContent>
              {templates.map((item) => (
                <SelectItem key={item.id} value={item.id}>
                  {item.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
        <Field label={t("mail.compose.subject")} className="md:col-span-2">
          <Input value={subject} onChange={(event) => setSubject(event.target.value)} />
        </Field>
        <Field label={t("mail.compose.body")} className="md:col-span-2">
          <MailHtmlEditor
            bodyText={bodyText}
            bodyHtml={bodyHtml}
            onBodyTextChange={setBodyText}
            onBodyHtmlChange={setBodyHtml}
            textLabel={t("mail.compose.bodyModeText")}
            htmlLabel={t("mail.compose.bodyModeHtml")}
            previewLabel={t("mail.compose.htmlPreview")}
            textPlaceholder={t("mail.compose.replyPlaceholder")}
            htmlPlaceholder={t("mail.compose.htmlPlaceholder")}
            emptyPreviewLabel={t("mail.preview.emptyBody")}
          />
        </Field>
        <div className="md:col-span-2">
          <OpenTrackingSwitch
            enabled={openTrackingEnabled}
            onEnabledChange={setOpenTrackingEnabled}
            disabled={sending}
          />
        </div>
      </div>
    </Modal>
    </>
  );
});

/**
 * Labeled field wrapper for mail forms.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
const Field = memo(function Field({
  label,
  children,
  className,
}: {
  label: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <label className={cn("space-y-1.5", className)}>
      <span className="text-[length:var(--text-xs)] text-muted-foreground">{label}</span>
      {children}
    </label>
  );
});

/**
 * Lightweight modal chrome for mail dialogs.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
const Modal = memo(function Modal({
  title,
  children,
  footer,
  onClose,
}: {
  title: string;
  children: ReactNode;
  footer: ReactNode;
  onClose: () => void;
}) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4">
      <div className="flex max-h-[85vh] w-full max-w-2xl flex-col overflow-hidden rounded-xl border border-border bg-dialog text-dialog-foreground shadow-lg">
        <div className="flex items-center justify-between border-b px-5 py-3">
          <h4 className="text-sm font-semibold">{title}</h4>
          <button
            type="button"
            className="text-muted-foreground hover:text-foreground"
            onClick={onClose}
            aria-label="close"
          >
            ✕
          </button>
        </div>
        <div className="flex-1 overflow-y-auto px-5 py-4">{children}</div>
        <div className="flex items-center justify-end gap-2 border-t px-5 py-3">{footer}</div>
      </div>
    </div>
  );
});

/**
 * Bind-mailbox dialog — IMAP / SMTP / auth sections (legacy `#bindModal`).
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
const AccountBindModal = memo(function AccountBindModal({
  onClose,
  onSaved,
}: {
  onClose: () => void;
  onSaved: (items: MailAccount[]) => void;
}) {
  const t = useT();
  const [form, setForm] = useState(EMPTY_ACCOUNT_FORM);
  const [saving, setSaving] = useState(false);

  async function save() {
    setSaving(true);
    try {
      const response = await mailAccountSave({
        label: form.label,
        from_address: form.fromAddress,
        from_name: form.fromName || undefined,
        smtp_host: form.smtpHost,
        smtp_port: Number(form.smtpPort || "465"),
        use_tls: form.useTls,
        username: form.username || form.fromAddress,
        password: form.password,
        imap_host: form.imapHost || undefined,
        imap_port: form.imapPort ? Number(form.imapPort) : undefined,
        imap_use_tls: form.imapUseTls,
        imap_sync_enabled: Boolean(form.imapHost.trim()),
      });
      toast.success(t("mail.statusAccountSaved"));
      onSaved(response.items);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  return (
    <Modal
      title={t("mail.accounts.bindTitle")}
      onClose={onClose}
      footer={
        <>
          <Button size="sm" variant="outline" onClick={onClose}>
            {t("mail.modal.cancel")}
          </Button>
          <Button size="sm" onClick={() => void save()} disabled={saving}>
            {saving ? t("mail.settings.savingAccount") : t("mail.settings.saveAccount")}
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        <div className="grid gap-3 md:grid-cols-2">
          <Field label={t("mail.settings.accountLabel")}>
            <Input
              value={form.label}
              onChange={(event) => setForm((current) => ({ ...current, label: event.target.value }))}
            />
          </Field>
          <Field label={t("mail.settings.fromAddress")}>
            <Input
              value={form.fromAddress}
              onChange={(event) =>
                setForm((current) => ({ ...current, fromAddress: event.target.value }))
              }
            />
          </Field>
          <Field label={t("mail.settings.fromName")} className="md:col-span-2">
            <Input
              value={form.fromName}
              onChange={(event) => setForm((current) => ({ ...current, fromName: event.target.value }))}
            />
          </Field>
        </div>

        <div className="space-y-3 border-t border-border/50 pt-3">
          <p className="text-xs font-medium text-foreground">{t("mail.settings.sectionImap")}</p>
          <div className="grid gap-3 md:grid-cols-[1fr_100px]">
            <Field label={t("mail.settings.imapHost")}>
              <Input
                value={form.imapHost}
                placeholder="imap.example.com"
                onChange={(event) =>
                  setForm((current) => ({ ...current, imapHost: event.target.value }))
                }
              />
            </Field>
            <Field label={t("mail.settings.imapPort")}>
              <Input
                value={form.imapPort}
                onChange={(event) =>
                  setForm((current) => ({ ...current, imapPort: event.target.value }))
                }
              />
            </Field>
          </div>
          <label className="inline-flex items-center gap-2 text-xs text-muted-foreground">
            <input
              type="checkbox"
              checked={form.imapUseTls}
              onChange={(event) =>
                setForm((current) => ({ ...current, imapUseTls: event.target.checked }))
              }
            />
            {t("mail.settings.imapTls")}
          </label>
        </div>

        <div className="space-y-3 border-t border-border/50 pt-3">
          <p className="text-xs font-medium text-foreground">{t("mail.settings.sectionSmtp")}</p>
          <div className="grid gap-3 md:grid-cols-[1fr_100px]">
            <Field label={t("mail.settings.smtpHost")}>
              <Input
                value={form.smtpHost}
                placeholder="smtp.example.com"
                onChange={(event) =>
                  setForm((current) => ({ ...current, smtpHost: event.target.value }))
                }
              />
            </Field>
            <Field label={t("mail.settings.smtpPort")}>
              <Input
                value={form.smtpPort}
                onChange={(event) =>
                  setForm((current) => ({ ...current, smtpPort: event.target.value }))
                }
              />
            </Field>
          </div>
          <label className="inline-flex items-center gap-2 text-xs text-muted-foreground">
            <input
              type="checkbox"
              checked={form.useTls}
              onChange={(event) =>
                setForm((current) => ({ ...current, useTls: event.target.checked }))
              }
            />
            {t("mail.settings.smtpTls")}
          </label>
        </div>

        <div className="space-y-3 border-t border-border/50 pt-3">
          <p className="text-xs font-medium text-foreground">{t("mail.settings.sectionAuth")}</p>
          <div className="grid gap-3 md:grid-cols-2">
            <Field label={t("mail.settings.username")}>
              <Input
                value={form.username}
                placeholder={form.fromAddress || "you@example.com"}
                onChange={(event) =>
                  setForm((current) => ({ ...current, username: event.target.value }))
                }
              />
            </Field>
            <Field label={t("mail.settings.password")}>
              <Input
                type="password"
                value={form.password}
                onChange={(event) =>
                  setForm((current) => ({ ...current, password: event.target.value }))
                }
              />
            </Field>
          </div>
        </div>
      </div>
    </Modal>
  );
});

/**
 * Create custom mail template dialog.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
const TemplateSaveModal = memo(function TemplateSaveModal({
  onClose,
  onSaved,
}: {
  onClose: () => void;
  onSaved: (items: MailTemplate[]) => void;
}) {
  const t = useT();
  const [form, setForm] = useState(EMPTY_TEMPLATE_FORM);
  const [saving, setSaving] = useState(false);

  async function save() {
    if (!form.name.trim() || !form.subjectTemplate.trim() || !form.bodyTextTemplate.trim()) {
      toast.error(t("mail.statusTemplateValidation"));
      return;
    }
    setSaving(true);
    try {
      const response = await mailTemplateSave({
        name: form.name.trim(),
        template_intent: form.templateIntent || "custom",
        subject_template: form.subjectTemplate,
        body_text_template: form.bodyTextTemplate,
        is_active: true,
        sort_order: 100,
      });
      toast.success(t("mail.statusTemplateSaved"));
      onSaved(response.items);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  return (
    <Modal
      title={t("mail.templates.modalTitle")}
      onClose={onClose}
      footer={
        <>
          <Button size="sm" variant="outline" onClick={onClose}>
            {t("mail.modal.cancel")}
          </Button>
          <Button size="sm" onClick={() => void save()} disabled={saving}>
            {saving ? t("mail.templates.saving") : t("mail.templates.save")}
          </Button>
        </>
      }
    >
      <div className="grid gap-3">
        <Field label={t("mail.templates.name")}>
          <Input
            value={form.name}
            onChange={(event) => setForm((current) => ({ ...current, name: event.target.value }))}
          />
        </Field>
        <Field label={t("mail.templates.intent")}>
          <Input
            value={form.templateIntent}
            onChange={(event) =>
              setForm((current) => ({ ...current, templateIntent: event.target.value }))
            }
          />
        </Field>
        <Field label={t("mail.templates.subject")}>
          <Input
            value={form.subjectTemplate}
            onChange={(event) =>
              setForm((current) => ({ ...current, subjectTemplate: event.target.value }))
            }
          />
        </Field>
        <Field label={t("mail.templates.body")}>
          <textarea
            value={form.bodyTextTemplate}
            onChange={(event) =>
              setForm((current) => ({ ...current, bodyTextTemplate: event.target.value }))
            }
            className="min-h-40 w-full rounded-[var(--radius-md)] border border-input bg-background px-3 py-2 text-[length:var(--text-sm)]"
          />
        </Field>
      </div>
    </Modal>
  );
});

/**
 * Manual inbound record dialog.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
const InboundRecordModal = memo(function InboundRecordModal({
  customers,
  initialCustomerId,
  onClose,
  onSaved,
}: {
  customers: CustomerProfile[];
  initialCustomerId: string;
  onClose: () => void;
  onSaved: () => void | Promise<void>;
}) {
  const t = useT();
  const [customerId, setCustomerId] = useState(initialCustomerId || customers[0]?.id || "");
  const [form, setForm] = useState(EMPTY_INBOUND_FORM);
  const [saving, setSaving] = useState(false);

  async function save() {
    if (!customerId || !form.fromAddress.trim() || !form.subject.trim()) {
      toast.error(t("mail.statusInboundValidation"));
      return;
    }
    setSaving(true);
    try {
      const response = await mailRecordInbound({
        customer_id: customerId,
        from_address: form.fromAddress.trim(),
        subject: form.subject.trim(),
        body_text: form.bodyText,
        received_at: String(Date.now()),
      });
      toast.success(t("mail.statusInboundRecorded", { id: response.message_id }));
      await onSaved();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  return (
    <Modal
      title={t("mail.inbound.modalTitle")}
      onClose={onClose}
      footer={
        <>
          <Button size="sm" variant="outline" onClick={onClose}>
            {t("mail.modal.cancel")}
          </Button>
          <Button size="sm" onClick={() => void save()} disabled={saving}>
            {saving ? t("mail.inbound.recording") : t("mail.inbound.record")}
          </Button>
        </>
      }
    >
      <div className="grid gap-3">
        <Field label={t("mail.compose.customer")}>
          <Select value={customerId} onValueChange={setCustomerId}>
            <SelectTrigger>
              <SelectValue placeholder={t("mail.compose.customerPlaceholder")} />
            </SelectTrigger>
            <SelectContent>
              {customers.map((item) => (
                <SelectItem key={item.id} value={item.id}>
                  {item.display_name || item.email}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
        <Field label={t("mail.inbound.from")}>
          <Input
            value={form.fromAddress}
            onChange={(event) =>
              setForm((current) => ({ ...current, fromAddress: event.target.value }))
            }
          />
        </Field>
        <Field label={t("mail.inbound.subject")}>
          <Input
            value={form.subject}
            onChange={(event) => setForm((current) => ({ ...current, subject: event.target.value }))}
          />
        </Field>
        <Field label={t("mail.inbound.body")}>
          <textarea
            value={form.bodyText}
            onChange={(event) => setForm((current) => ({ ...current, bodyText: event.target.value }))}
            className="min-h-32 w-full rounded-[var(--radius-md)] border border-input bg-background px-3 py-2 text-[length:var(--text-sm)]"
          />
        </Field>
      </div>
    </Modal>
  );
});

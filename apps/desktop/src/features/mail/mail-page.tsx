/**
 * Mail workspace — account rail, customer list, preview/reply.
 * Layout mirrors email-agent legacy mail tab, adapted for OpenDesk M2 APIs.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */

import type { ReactNode } from "react";
import { memo, useEffect, useMemo, useState } from "react";
import { Mail, Plus, RefreshCw } from "@desk/ui/icons";
import { useT } from "../../i18n";
import {
  Button,
  Input,
  LoadingState,
  PageScaffold,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  WorkspaceSplit,
  WorkspaceSplitPane,
  WorkspaceSplitRow,
  WorkspaceSplitTitle,
  WorkspaceSplitToolbar,
  cn,
  toast,
} from "@desk/ui";
import {
  customerList,
  mailAccountList,
  mailAccountSave,
  mailRecordInbound,
  mailSend,
  mailTemplateApply,
  mailTemplateList,
  type CustomerProfile,
  type MailAccount,
  type MailTemplate,
} from "@desk/platform";

/** Stable toast id so StrictMode remounts update one toast instead of stacking two. */
const MAIL_BOOTSTRAP_TOAST_ID = "mail-bootstrap";

const EMPTY_ACCOUNT_FORM = {
  label: "",
  fromAddress: "",
  fromName: "",
  smtpHost: "",
  smtpPort: "465",
  useTls: "true",
  username: "",
  password: "",
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
 * @returns Mail workspace node
 */
export function MailPage() {
  const t = useT();
  const [customers, setCustomers] = useState<CustomerProfile[]>([]);
  const [templates, setTemplates] = useState<MailTemplate[]>([]);
  const [accounts, setAccounts] = useState<MailAccount[]>([]);
  const [customerId, setCustomerId] = useState("");
  const [templateId, setTemplateId] = useState("");
  const [accountFilterId, setAccountFilterId] = useState<string | null>(null);
  const [accountId, setAccountId] = useState("");
  const [customerQuery, setCustomerQuery] = useState("");
  const [subject, setSubject] = useState("");
  const [bodyText, setBodyText] = useState("");
  const [sending, setSending] = useState(false);
  const [loading, setLoading] = useState(true);
  const [accountModalOpen, setAccountModalOpen] = useState(false);
  const [inboundModalOpen, setInboundModalOpen] = useState(false);

  const selectedCustomer = useMemo(
    () => customers.find((item) => item.id === customerId) ?? null,
    [customerId, customers],
  );

  const filteredCustomers = useMemo(() => {
    const kw = customerQuery.trim().toLowerCase();
    if (!kw) return customers;
    return customers.filter((item) => {
      const name = (item.display_name ?? "").toLowerCase();
      const email = (item.email ?? "").toLowerCase();
      return name.includes(kw) || email.includes(kw);
    });
  }, [customerQuery, customers]);

  async function loadMailWorkspace(signal?: { cancelled: boolean }) {
    setLoading(true);
    toast.loading(t("mail.statusLoading"), { id: MAIL_BOOTSTRAP_TOAST_ID });
    try {
      const [customerResponse, templateResponse, accountResponse] = await Promise.all([
        customerList(),
        mailTemplateList(),
        mailAccountList(),
      ]);
      if (signal?.cancelled) {
        return;
      }
      setCustomers(customerResponse.items);
      setTemplates(templateResponse.items);
      setAccounts(accountResponse.items);
      setCustomerId((current) => current || customerResponse.items[0]?.id || "");
      setTemplateId((current) => current || templateResponse.items[0]?.id || "");
      setAccountId((current) => current || accountResponse.items[0]?.id || "");
      toast.success(t("mail.statusReady"), { id: MAIL_BOOTSTRAP_TOAST_ID });
    } catch (error) {
      if (signal?.cancelled) {
        return;
      }
      toast.error(error instanceof Error ? error.message : String(error), {
        id: MAIL_BOOTSTRAP_TOAST_ID,
      });
    } finally {
      if (!signal?.cancelled) {
        setLoading(false);
      }
    }
  }

  useEffect(() => {
    const signal = { cancelled: false };
    toast.loading(t("mail.statusLoading"), { id: MAIL_BOOTSTRAP_TOAST_ID });
    void Promise.all([customerList(), mailTemplateList(), mailAccountList()])
      .then(([customerResponse, templateResponse, accountResponse]) => {
        if (signal.cancelled) {
          return;
        }
        setCustomers(customerResponse.items);
        setTemplates(templateResponse.items);
        setAccounts(accountResponse.items);
        setCustomerId((current) => current || customerResponse.items[0]?.id || "");
        setTemplateId((current) => current || templateResponse.items[0]?.id || "");
        setAccountId((current) => current || accountResponse.items[0]?.id || "");
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
  }, []);

  function selectAccountFilter(id: string | null) {
    setAccountFilterId(id);
    if (id) {
      setAccountId(id);
    }
  }

  async function applyTemplate() {
    if (!customerId || !templateId) {
      toast.error(t("mail.statusSelectFirst"));
      return;
    }
    try {
      const response = await mailTemplateApply({
        customer_id: customerId,
        template_id: templateId,
        account_id: accountId || undefined,
      });
      setSubject(response.subject);
      setBodyText(response.body_text);
      toast.success(t("mail.statusTemplateApplied"));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function sendOutbound() {
    if (!customerId || !templateId || !accountId || !subject.trim() || !bodyText.trim()) {
      toast.error(t("mail.statusSendValidation"));
      return;
    }
    setSending(true);
    try {
      const response = await mailSend({
        customer_id: customerId,
        account_id: accountId,
        template_id: templateId,
        subject,
        body_text: bodyText,
      });
      toast.success(t("mail.statusSent", { id: response.message_id }));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSending(false);
    }
  }

  return (
    <PageScaffold fill containerPadding="none">
      <WorkspaceSplit className="min-h-0 flex-1 border-0" defaultStartWidth={168} minStartWidth={140} maxStartWidth={220}>
        {/* Left: account rail */}
        <WorkspaceSplitPane
          side="start"
          header={
            <WorkspaceSplitToolbar className="justify-between px-3 py-2">
              <WorkspaceSplitTitle className="text-xs">{t("mail.accounts.title")}</WorkspaceSplitTitle>
              <Button
                size="sm"
                variant="ghost"
                className="size-7 p-0"
                disabled={loading}
                onClick={() => void loadMailWorkspace()}
              >
                <RefreshCw className={cn("size-3.5", loading && "animate-spin")} />
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

        {/* Center + Right: customer list | preview/reply */}
        <WorkspaceSplitPane scroll={false}>
          <WorkspaceSplit
            className="h-full min-h-0 border-0"
            defaultStartWidth={340}
            minStartWidth={280}
            maxStartWidth={480}
          >
            {/* Customer list */}
            <WorkspaceSplitPane
              side="start"
              header={
                <WorkspaceSplitToolbar className="flex-col items-stretch gap-2 px-3 py-2">
                  <div className="flex items-center justify-between">
                    <WorkspaceSplitTitle className="text-xs">{t("mail.list.title")}</WorkspaceSplitTitle>
                    <span className="text-[10px] text-muted-foreground">
                      {t("mail.list.total", { count: filteredCustomers.length })}
                    </span>
                  </div>
                  <Input
                    value={customerQuery}
                    onChange={(event) => setCustomerQuery(event.target.value)}
                    placeholder={t("mail.list.searchPlaceholder")}
                    className="h-7 text-xs"
                  />
                </WorkspaceSplitToolbar>
              }
            >
              {loading ? (
                <LoadingState label={t("mail.statusLoading")} />
              ) : filteredCustomers.length === 0 ? (
                <p className="px-4 py-10 text-center text-xs text-muted-foreground">{t("mail.list.empty")}</p>
              ) : (
                filteredCustomers.map((item) => (
                  <WorkspaceSplitRow
                    key={item.id}
                    active={customerId === item.id}
                    className="px-3 py-2.5"
                    onClick={() => setCustomerId(item.id)}
                  >
                    <div className="truncate text-sm font-medium">
                      {item.display_name || item.email}
                    </div>
                    <div className="mt-0.5 truncate text-xs text-muted-foreground">{item.email}</div>
                    {item.outreach_stage ? (
                      <span className="mt-1 inline-block rounded bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">
                        {item.outreach_stage}
                      </span>
                    ) : null}
                  </WorkspaceSplitRow>
                ))
              )}
            </WorkspaceSplitPane>

            {/* Preview / reply */}
            <WorkspaceSplitPane
              header={
                <WorkspaceSplitToolbar className="justify-between px-4 py-2">
                  <div className="min-w-0">
                    <WorkspaceSplitTitle className="truncate text-sm">
                      {selectedCustomer?.display_name || selectedCustomer?.email || t("mail.preview.emptyTitle")}
                    </WorkspaceSplitTitle>
                    {selectedCustomer ? (
                      <p className="truncate text-xs text-muted-foreground">{selectedCustomer.email}</p>
                    ) : null}
                  </div>
                  <div className="flex items-center gap-2">
                    <Button size="sm" variant="outline" onClick={() => setInboundModalOpen(true)} disabled={!customerId}>
                      {t("mail.inbound.record")}
                    </Button>
                  </div>
                </WorkspaceSplitToolbar>
              }
            >
              {loading ? (
                <LoadingState className="h-full" label={t("mail.statusLoading")} size="lg" />
              ) : !selectedCustomer ? (
                <div className="flex h-full items-center justify-center px-6 text-center text-sm text-muted-foreground">
                  {t("mail.preview.empty")}
                </div>
              ) : (
                <div className="flex h-full flex-col gap-4 p-4">
                  <div className="grid gap-3 md:grid-cols-2">
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

                  <Field label={t("mail.compose.body")} className="flex min-h-0 flex-1 flex-col">
                    <textarea
                      value={bodyText}
                      onChange={(event) => setBodyText(event.target.value)}
                      className="min-h-48 w-full flex-1 rounded-[var(--radius-md)] border border-input bg-background px-3 py-2 text-[length:var(--text-sm)]"
                    />
                  </Field>

                  <div className="flex items-center gap-2 border-t border-border/50 pt-3">
                    <Button size="sm" onClick={() => void sendOutbound()} disabled={sending}>
                      {sending ? t("mail.compose.sending") : t("mail.compose.send")}
                    </Button>
                  </div>
                </div>
              )}
            </WorkspaceSplitPane>
          </WorkspaceSplit>
        </WorkspaceSplitPane>
      </WorkspaceSplit>

      {accountModalOpen ? (
        <AccountBindModal
          onClose={() => setAccountModalOpen(false)}
          onSaved={(items) => {
            setAccounts(items);
            setAccountId(items[0]?.id ?? "");
            setAccountModalOpen(false);
          }}
        />
      ) : null}

      {inboundModalOpen ? (
        <InboundRecordModal
          customerId={customerId}
          onClose={() => setInboundModalOpen(false)}
        />
      ) : null}
    </PageScaffold>
  );
}

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
      <div className="flex max-h-[85vh] w-full max-w-2xl flex-col overflow-hidden rounded-xl border bg-card shadow-xl">
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
 * Bind-mailbox dialog — owns its draft form state so the mail page does not re-render on each keystroke.
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
        use_tls: form.useTls === "true",
        username: form.username,
        password: form.password,
        imap_sync_enabled: false,
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
        <Field label={t("mail.settings.fromName")}>
          <Input
            value={form.fromName}
            onChange={(event) => setForm((current) => ({ ...current, fromName: event.target.value }))}
          />
        </Field>
        <Field label={t("mail.settings.smtpHost")}>
          <Input
            value={form.smtpHost}
            onChange={(event) => setForm((current) => ({ ...current, smtpHost: event.target.value }))}
          />
        </Field>
        <Field label={t("mail.settings.smtpPort")}>
          <Input
            value={form.smtpPort}
            onChange={(event) => setForm((current) => ({ ...current, smtpPort: event.target.value }))}
          />
        </Field>
        <Field label={t("mail.settings.username")}>
          <Input
            value={form.username}
            onChange={(event) => setForm((current) => ({ ...current, username: event.target.value }))}
          />
        </Field>
        <Field label={t("mail.settings.password")} className="md:col-span-2">
          <Input
            type="password"
            value={form.password}
            onChange={(event) => setForm((current) => ({ ...current, password: event.target.value }))}
          />
        </Field>
      </div>
    </Modal>
  );
});

/**
 * Inbound record dialog — local draft state isolated from the mail workspace list.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
const InboundRecordModal = memo(function InboundRecordModal({
  customerId,
  onClose,
}: {
  customerId: string;
  onClose: () => void;
}) {
  const t = useT();
  const [form, setForm] = useState(EMPTY_INBOUND_FORM);
  const [recording, setRecording] = useState(false);

  async function record() {
    if (!customerId || !form.fromAddress.trim() || !form.subject.trim()) {
      toast.error(t("mail.statusInboundValidation"));
      return;
    }
    setRecording(true);
    try {
      const response = await mailRecordInbound({
        customer_id: customerId,
        from_address: form.fromAddress,
        subject: form.subject,
        body_text: form.bodyText,
        received_at: new Date().toISOString(),
      });
      toast.success(t("mail.statusInboundRecorded", { id: response.message_id }));
      onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setRecording(false);
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
          <Button size="sm" onClick={() => void record()} disabled={recording}>
            {recording ? t("mail.inbound.recording") : t("mail.inbound.record")}
          </Button>
        </>
      }
    >
      <div className="grid gap-3">
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
            onChange={(event) =>
              setForm((current) => ({ ...current, bodyText: event.target.value }))
            }
            className="min-h-32 w-full rounded-[var(--radius-md)] border border-input bg-background px-3 py-2 text-[length:var(--text-sm)]"
          />
        </Field>
      </div>
    </Modal>
  );
});

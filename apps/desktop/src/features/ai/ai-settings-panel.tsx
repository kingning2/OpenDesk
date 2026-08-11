/**
 * AI 平台与账号配置面板(设置弹窗内)。
 *
 * 平台为内置(DeepSeek / Ollama),用户只管理账号。
 *
 * @author coisini
 * @created 2026-08-11
 */

import { useEffect, useState } from "react";
import { ChevronDown, ChevronRight, Pencil, Plus, Trash2 } from "@desk/ui/icons";
import { Button, Card, Dialog, DialogContent, Input } from "@desk/ui";
import type { AiAccount } from "@desk/contracts";
import { aiTestApiKey } from "@desk/platform/ipc/ai";
import deepseekLogo from "../../assets/deepseek.svg";
import ollamaLogo from "../../assets/ollama.svg";
import { useT } from "../../i18n";
import type { BuiltInProvider } from "./builtin-providers";
import { useAiConfigStore, type AiAccountInput } from "./use-ai-config";

function maskKey(key: string): string {
  if (!key) return "—";
  if (key.length <= 8) return "••••••••";
  return `${key.slice(0, 3)}••••••••${key.slice(-4)}`;
}

function toError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/**
 * 添加/编辑账号对话框。
 *
 * 提供 API Key 测试按钮（仅 DeepSeek 平台），通过余额接口校验可用性。
 *
 * @author coisini
 * @created 2026-08-11
 */
function AccountDialog({
  open,
  provider,
  account,
  onClose,
}: {
  open: boolean;
  provider: BuiltInProvider | null;
  account: AiAccount | null;
  onClose: () => void;
}) {
  const t = useT();
  const addAccount = useAiConfigStore((state) => state.addAccount);
  const updateAccount = useAiConfigStore((state) => state.updateAccount);

  const [name, setName] = useState(account?.name ?? "");
  const [apiKey, setApiKey] = useState(account?.api_key ?? "");
  const [defaultModel, setDefaultModel] = useState(account?.default_model ?? "");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{
    ok: boolean;
    message: string;
  } | null>(null);

  async function handleTest() {
    const trimmedKey = apiKey.trim();
    if (!trimmedKey) {
      setError(t("ai.errorApiKeyRequired"));
      return;
    }
    setTesting(true);
    setTestResult(null);
    setError(null);
    try {
      setTestResult(await aiTestApiKey(provider?.base_url ?? "", trimmedKey));
    } catch (caught) {
      setError(toError(caught));
    } finally {
      setTesting(false);
    }
  }

  async function handleSubmit() {
    const trimmedName = name.trim();
    const trimmedKey = apiKey.trim();
    const trimmedModel = defaultModel.trim();
    if (!trimmedName) {
      setError(t("ai.errorNameRequired"));
      return;
    }
    if (!trimmedKey) {
      setError(t("ai.errorApiKeyRequired"));
      return;
    }
    setSaving(true);
    setError(null);
    try {
      const input: AiAccountInput = {
        provider_id: provider?.id ?? "",
        name: trimmedName,
        api_key: trimmedKey,
        default_model: trimmedModel || undefined,
      };
      if (account) {
        await updateAccount(account.id, input);
      } else {
        await addAccount(input);
      }
      onClose();
    } catch (caught) {
      setError(toError(caught));
    } finally {
      setSaving(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={(next) => !next && onClose()}>
      <DialogContent
        title={account ? t("ai.editAccountTitle") : t("ai.addAccountTitle")}
        footer={
          <>
            <Button variant="ghost" disabled={saving} onClick={onClose}>
              {t("ai.cancel")}
            </Button>
            <Button disabled={saving} onClick={() => void handleSubmit()}>
              {t("ai.save")}
            </Button>
          </>
        }
      >
        <div className="flex flex-col gap-4">
          <div className="flex flex-col gap-2">
            <label
              htmlFor="ai-account-name"
              className="text-[length:var(--text-sm)] font-medium text-foreground"
            >
              {t("ai.accountName")}
            </label>
            <Input
              id="ai-account-name"
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder={t("ai.accountNamePlaceholder")}
            />
          </div>
          <div className="flex flex-col gap-2">
            <label
              htmlFor="ai-account-key"
              className="text-[length:var(--text-sm)] font-medium text-foreground"
            >
              {t("ai.apiKey")}
            </label>
            <div className="flex items-center gap-2">
              <Input
                id="ai-account-key"
                type="password"
                value={apiKey}
                onChange={(event) => {
                  setApiKey(event.target.value);
                  setTestResult(null);
                }}
                placeholder={t("ai.apiKeyPlaceholder")}
                className="min-w-0 flex-1"
              />
              {provider?.kind === "deepseek" ? (
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  disabled={testing}
                  onClick={() => void handleTest()}
                >
                  {testing ? t("ai.testing") : t("ai.testKey")}
                </Button>
              ) : null}
            </div>
            {testResult ? (
              <p
                className={
                  testResult.ok
                    ? "text-[length:var(--text-sm)] text-green-600 dark:text-green-400"
                    : "text-[length:var(--text-sm)] text-red-600 dark:text-red-400"
                }
              >
                {testResult.ok ? t("ai.testOk") : t("ai.testFail")}
              </p>
            ) : null}
          </div>
          <div className="flex flex-col gap-2">
            <label
              htmlFor="ai-account-model"
              className="text-[length:var(--text-sm)] font-medium text-foreground"
            >
              {t("ai.defaultModel")}
            </label>
            <Input
              id="ai-account-model"
              value={defaultModel}
              onChange={(event) => setDefaultModel(event.target.value)}
              placeholder={t("ai.defaultModelPlaceholder")}
            />
          </div>
          {error ? (
            <p className="text-[length:var(--text-sm)] text-red-600 dark:text-red-400">
              {error}
            </p>
          ) : null}
        </div>
      </DialogContent>
    </Dialog>
  );
}

/**
 * 删除确认对话框。
 *
 * @author coisini
 * @created 2026-08-11
 */
interface ConfirmState {
  title: string;
  description: string;
  confirmLabel: string;
  onConfirm: () => Promise<void>;
}

function ConfirmDialog({
  state,
  onClose,
}: {
  state: ConfirmState | null;
  onClose: () => void;
}) {
  const t = useT();
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleConfirm() {
    if (!state) return;
    setSaving(true);
    setError(null);
    try {
      await state.onConfirm();
      onClose();
    } catch (caught) {
      setError(toError(caught));
    } finally {
      setSaving(false);
    }
  }

  return (
    <Dialog open={state !== null} onOpenChange={(next) => !next && onClose()}>
      <DialogContent
        className="max-w-sm"
        title={state?.title}
        description={state?.description}
        footer={
          <>
            <Button variant="ghost" disabled={saving} onClick={onClose}>
              {t("ai.cancel")}
            </Button>
            <Button
              disabled={saving}
              className="text-red-600 dark:text-red-400"
              variant="outline"
              onClick={() => void handleConfirm()}
            >
              {state?.confirmLabel}
            </Button>
          </>
        }
      >
        {error ? (
          <p className="text-[length:var(--text-sm)] text-red-600 dark:text-red-400">
            {error}
          </p>
        ) : null}
      </DialogContent>
    </Dialog>
  );
}

/**
 * 平台品牌 Logo 映射。
 *
 * Ollama 为纯黑 Logo,暗色主题下反色显示,避免不可见。
 */
const providerLogos: Record<string, string> = {
  deepseek: deepseekLogo,
  ollama: ollamaLogo,
};

function ProviderLogo({ provider }: { provider: BuiltInProvider }) {
  return (
    <img
      src={providerLogos[provider.id]}
      alt=""
      aria-hidden
      className={`h-4 w-auto shrink-0${
        provider.id === "ollama" ? " dark:invert" : ""
      }`}
    />
  );
}

/**
 * 单个内置平台卡片(展开后显示账号列表与接口地址)。
 *
 * 无账号平台（如 Ollama）只展示接口地址与可用提示，不显示账号管理。
 *
 * @author coisini
 * @created 2026-08-11
 */
function ProviderCard({
  provider,
  accounts,
  expanded,
  onToggle,
  onAddAccount,
  onEditAccount,
  onDeleteAccount,
}: {
  provider: BuiltInProvider;
  accounts: AiAccount[];
  expanded: boolean;
  onToggle: () => void;
  onAddAccount: () => void;
  onEditAccount: (account: AiAccount) => void;
  onDeleteAccount: (account: AiAccount) => void;
}) {
  const t = useT();
  const setProviderDefaultModel = useAiConfigStore(
    (state) => state.setProviderDefaultModel,
  );
  const [modelDraft, setModelDraft] = useState(provider.default_model ?? "");
  const [modelError, setModelError] = useState<string | null>(null);
  const kindLabel =
    provider.kind === "deepseek"
      ? t("ai.kindDeepseek")
      : t("ai.kindOpenAICompatible");

  async function saveModel() {
    const trimmed = modelDraft.trim();
    if (trimmed === (provider.default_model ?? "")) return;
    setModelError(null);
    try {
      await setProviderDefaultModel(provider.id, trimmed);
    } catch (caught) {
      setModelError(toError(caught));
    }
  }

  return (
    <Card className="w-full">
      <div className="flex items-center gap-3 px-4 py-3">
        <button
          type="button"
          onClick={onToggle}
          className="flex min-w-0 flex-1 cursor-pointer items-center gap-2.5 text-left"
        >
          {expanded ? (
            <ChevronDown className="size-4 shrink-0 opacity-60" aria-hidden />
          ) : (
            <ChevronRight className="size-4 shrink-0 opacity-60" aria-hidden />
          )}
          <ProviderLogo provider={provider} />
          <span className="truncate font-medium text-foreground">{provider.name}</span>
          <span className="shrink-0 rounded-full border border-border/70 px-2 py-0.5 text-[length:var(--text-xs)] text-muted-foreground">
            {kindLabel}
          </span>
          {!provider.authless ? (
            <span className="shrink-0 text-[length:var(--text-xs)] text-muted-foreground">
              {accounts.length} {t("ai.accounts")}
            </span>
          ) : null}
        </button>
        {!provider.authless ? (
          <div className="flex shrink-0 items-center gap-1">
            <Button size="sm" variant="ghost" onClick={onAddAccount}>
              <Plus className="size-3.5" aria-hidden />
              {t("ai.addAccount")}
            </Button>
          </div>
        ) : null}
      </div>
      {expanded ? (
        <div className="border-t border-border/70 px-4 py-3">
          {provider.base_url ? (
            <p className="mb-2 break-all font-mono text-[length:var(--text-xs)] text-muted-foreground">
              {provider.base_url}
            </p>
          ) : null}
          {provider.authless ? (
            <div className="flex flex-col gap-3">
              <div className="flex flex-col gap-2">
                <label
                  htmlFor={`ai-model-${provider.id}`}
                  className="text-[length:var(--text-sm)] font-medium text-foreground"
                >
                  {t("ai.defaultModel")}
                </label>
                <Input
                  id={`ai-model-${provider.id}`}
                  value={modelDraft}
                  onChange={(event) => setModelDraft(event.target.value)}
                  onBlur={() => void saveModel()}
                  placeholder={t("ai.defaultModelPlaceholder")}
                />
              </div>
              <p className="text-[length:var(--text-xs)] text-muted-foreground">
                {t("ai.authlessHint")}
              </p>
              {modelError ? (
                <p className="text-[length:var(--text-sm)] text-red-600 dark:text-red-400">
                  {modelError}
                </p>
              ) : null}
            </div>
          ) : accounts.length === 0 ? (
            <p className="text-[length:var(--text-sm)] text-muted-foreground">
              {t("ai.emptyAccounts")}
            </p>
          ) : (
            <ul className="flex flex-col gap-1">
              {accounts.map((account) => (
                <li
                  key={account.id}
                  className="flex items-center gap-3 rounded-[var(--radius-md)] px-2 py-1.5 transition-colors [@media(hover:hover)_and_(pointer:fine)]:hover:bg-muted/60"
                >
                  <span className="min-w-0 flex-1 truncate text-[length:var(--text-sm)] text-foreground">
                    {account.name}
                  </span>
                  <span className="shrink-0 font-mono text-[length:var(--text-xs)] text-muted-foreground">
                    {maskKey(account.api_key)}
                  </span>
                  {account.default_model ? (
                    <span className="shrink-0 text-[length:var(--text-xs)] text-muted-foreground">
                      {account.default_model}
                    </span>
                  ) : null}
                  <Button
                    size="icon"
                    variant="ghost"
                    onClick={() => onEditAccount(account)}
                    aria-label={t("ai.edit")}
                  >
                    <Pencil className="size-3.5" aria-hidden />
                  </Button>
                  <Button
                    size="icon"
                    variant="ghost"
                    onClick={() => onDeleteAccount(account)}
                    aria-label={t("ai.delete")}
                  >
                    <Trash2 className="size-3.5" aria-hidden />
                  </Button>
                </li>
              ))}
            </ul>
          )}
        </div>
      ) : null}
    </Card>
  );
}

/**
 * AI 设置面板入口。
 *
 * @author coisini
 * @created 2026-08-11
 *
 * @returns 面板节点
 */
export function AiSettingsPanel() {
  const t = useT();
  const providers = useAiConfigStore((state) => state.providers);
  const accounts = useAiConfigStore((state) => state.accounts);
  const loading = useAiConfigStore((state) => state.loading);
  const loaded = useAiConfigStore((state) => state.loaded);
  const loadError = useAiConfigStore((state) => state.error);
  const load = useAiConfigStore((state) => state.load);
  const removeAccount = useAiConfigStore((state) => state.removeAccount);

  const [expandedProviderId, setExpandedProviderId] = useState<string | null>(null);
  /** 递增触发对话框重挂载,以 props 初始化表单。 */
  const [dialogSeq, setDialogSeq] = useState(0);
  const [accountDialog, setAccountDialog] = useState<{
    provider: BuiltInProvider;
    account: AiAccount | null;
  } | null>(null);
  const [confirm, setConfirm] = useState<ConfirmState | null>(null);

  useEffect(() => {
    if (!loaded && !loading) {
      void load();
    }
  }, [loaded, loading, load]);

  function openDeleteAccount(account: AiAccount) {
    setDialogSeq((seq) => seq + 1);
    setConfirm({
      title: t("ai.deleteAccountTitle"),
      description: t("ai.deleteAccountDesc"),
      confirmLabel: t("ai.delete"),
      onConfirm: () => removeAccount(account.id),
    });
  }

  return (
    <div className="flex w-full flex-col gap-5">
      <p className="max-w-md text-[length:var(--text-sm)] leading-relaxed text-muted-foreground">
        {t("ai.hint")}
      </p>

      {loadError ? (
        <p className="text-[length:var(--text-sm)] text-red-600 dark:text-red-400">
          {loadError}
        </p>
      ) : null}

      <div className="flex flex-col gap-3">
        {providers.map((provider) => (
          <ProviderCard
            key={provider.id}
            provider={provider}
            accounts={accounts.filter(
              (account) => account.provider_id === provider.id,
            )}
            expanded={expandedProviderId === provider.id}
            onToggle={() =>
              setExpandedProviderId((current) =>
                current === provider.id ? null : provider.id,
              )
            }
            onAddAccount={() => {
              setDialogSeq((seq) => seq + 1);
              setAccountDialog({ provider, account: null });
            }}
            onEditAccount={(account) => {
              setDialogSeq((seq) => seq + 1);
              setAccountDialog({ provider, account });
            }}
            onDeleteAccount={openDeleteAccount}
          />
        ))}
      </div>

      <AccountDialog
        key={`account-${dialogSeq}`}
        open={accountDialog !== null}
        provider={accountDialog?.provider ?? null}
        account={accountDialog?.account ?? null}
        onClose={() => setAccountDialog(null)}
      />
      <ConfirmDialog
        key={`confirm-${dialogSeq}`}
        state={confirm}
        onClose={() => setConfirm(null)}
      />
    </div>
  );
}

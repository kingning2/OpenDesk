/**
 * 设置弹窗 — 左侧分类导航 + 右侧详情（双栏留白布局）。
 *
 * @author coisini
 * @created 2026-07-21
 */

import { useState } from "react";
import { useNavigate } from "react-router";
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
  IconButton,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  cn,
} from "@desk/ui";
import { Languages, X, Youtube, type LucideIcon } from "@desk/ui/icons";
import { useI18n } from "../../i18n";
import type { AppLocale } from "../../i18n";
import { useYoutubeApiKeySettings } from "./use-youtube-api-key-settings";

/** 设置侧栏分区 id。 */
type SettingsSectionId = "language" | "youtube";

/**
 * `SettingsDialog` 属性。
 *
 * @author coisini
 * @created 2026-07-21
 */
export interface SettingsDialogProps {
  /** 是否打开。 */
  open: boolean;
  /** 打开状态变更。 */
  onOpenChange: (open: boolean) => void;
}

/**
 * 侧栏导航项。
 *
 * @author coisini
 * @created 2026-07-21
 */
interface NavItem {
  id: SettingsSectionId;
  labelKey: string;
  icon: LucideIcon;
}

/**
 * 侧栏分组。
 *
 * @author coisini
 * @created 2026-07-21
 */
interface NavGroup {
  labelKey: string;
  items: NavItem[];
}

const NAV_GROUPS: NavGroup[] = [
  {
    labelKey: "settings.navPreferences",
    items: [{ id: "language", labelKey: "settings.navLanguage", icon: Languages }],
  },
  {
    labelKey: "settings.navIntegrations",
    items: [{ id: "youtube", labelKey: "settings.navYoutube", icon: Youtube }],
  },
];

/**
 * 应用设置弹窗（双栏）。
 *
 * @author coisini
 * @created 2026-07-21
 *
 * @param props - 见 {@link SettingsDialogProps}
 * @returns 设置 Dialog 节点
 */
export function SettingsDialog({ open, onOpenChange }: SettingsDialogProps) {
  const navigate = useNavigate();
  const { t, locale, setLocale, locales, localeLabels } = useI18n();
  const {
    apiKey,
    setApiKey,
    loading,
    saving,
    savedMessage,
    error,
    save,
    discard,
    dirty: youtubeDirty,
    configured,
  } = useYoutubeApiKeySettings();
  const [section, setSection] = useState<SettingsSectionId>("language");
  const [draftLocale, setDraftLocale] = useState<AppLocale>(locale);
  const [localeSavedMessage, setLocaleSavedMessage] = useState("");
  const [confirmExit, setConfirmExit] = useState(false);
  const [exitPendingAction, setExitPendingAction] = useState<"close" | "crawler" | null>(
    null,
  );
  const [wasOpen, setWasOpen] = useState(open);

  if (open !== wasOpen) {
    setWasOpen(open);
    if (open) {
      setDraftLocale(locale);
      setLocaleSavedMessage("");
      setConfirmExit(false);
      setExitPendingAction(null);
    }
  }

  const localeDirty = draftLocale !== locale;
  const isDirty = localeDirty || youtubeDirty;
  /**
   * 保存当前脏字段（语言草稿 + YouTube 密钥）。
   *
   * @author coisini
   * @created 2026-07-21
   *
   * @returns 是否全部保存成功
   */
  async function saveAll(): Promise<boolean> {
    try {
      if (localeDirty) {
        setLocale(draftLocale);
        setLocaleSavedMessage(t("settings.saved"));
      }
      if (youtubeDirty) {
        await save();
      }
      return true;
    } catch {
      return false;
    }
  }

  /**
   * 放弃未保存修改。
   *
   * @author coisini
   * @created 2026-07-21
   */
  function discardAll() {
    setDraftLocale(locale);
    setLocaleSavedMessage("");
    discard();
  }

  /**
   * 真正关闭弹窗（或跳转采集），并清理确认态。
   *
   * @author coisini
   * @created 2026-07-21
   *
   * @param action - 关闭目标
   */
  function finishExit(action: "close" | "crawler") {
    setConfirmExit(false);
    setExitPendingAction(null);
    onOpenChange(false);
    if (action === "crawler") {
      navigate("/features/crawler");
    }
  }

  /**
   * 请求退出；有未保存更改时弹出确认。
   *
   * @author coisini
   * @created 2026-07-21
   *
   * @param action - 关闭或去采集页
   */
  function requestExit(action: "close" | "crawler") {
    if (isDirty) {
      setExitPendingAction(action);
      setConfirmExit(true);
      return;
    }
    finishExit(action);
  }

  /**
   * Radix open 变更：拦截关闭以做未保存确认。
   *
   * @author coisini
   * @created 2026-07-21
   *
   * @param next - 下一 open 状态
   */
  function handleOpenChange(next: boolean) {
    if (next) {
      onOpenChange(true);
      return;
    }
    requestExit("close");
  }

  const sectionTitle =
    section === "language" ? t("settings.language") : t("settings.youtubeTitle");

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent
        className="h-[min(780px,92vh)] w-[min(1100px,96vw)] max-w-none gap-0 p-0"
        closeLabel={t("settings.close")}
        dismissOnOutsidePress={false}
        showClose={false}
        onEscapeKeyDown={(event) => {
          event.preventDefault();
          requestExit("close");
        }}
      >
        <DialogTitle className="sr-only">{t("settings.title")}</DialogTitle>
        <DialogDescription className="sr-only">{t("settings.subtitle")}</DialogDescription>

        <IconButton
          label={t("settings.close")}
          className="absolute right-3 top-3 z-20"
          onClick={() => requestExit("close")}
        >
          <X className="size-3.5" aria-hidden />
        </IconButton>

        <div className="relative flex h-full min-h-0">
          <aside className="flex w-56 shrink-0 flex-col border-r border-border/70 bg-muted/25 py-5">
            <div className="px-4 pb-4">
              <p className="text-[length:var(--text-sm)] font-semibold tracking-tight text-foreground">
                {t("settings.title")}
              </p>
            </div>

            <nav className="flex min-h-0 flex-1 flex-col gap-5 overflow-y-auto px-2.5">
              {NAV_GROUPS.map((group) => (
                <div key={group.labelKey} className="flex flex-col gap-1">
                  <p className="px-2.5 pb-1 text-[length:var(--text-xs)] font-medium text-muted-foreground">
                    {t(group.labelKey)}
                  </p>
                  {group.items.map((item) => {
                    const Icon = item.icon;
                    const active = section === item.id;
                    return (
                      <button
                        key={item.id}
                        type="button"
                        onClick={() => setSection(item.id)}
                        className={cn(
                          "flex cursor-pointer items-center gap-2.5 rounded-[var(--radius-md)] px-2.5 py-2",
                          "text-left text-[length:var(--text-sm)] transition-[color,background-color,transform]",
                          "duration-150 ease-[cubic-bezier(0.23,1,0.32,1)] active:scale-[0.98]",
                          active
                            ? "bg-muted text-foreground"
                            : "text-muted-foreground [@media(hover:hover)_and_(pointer:fine)]:hover:bg-muted/60 [@media(hover:hover)_and_(pointer:fine)]:hover:text-foreground",
                        )}
                      >
                        <Icon className="size-4 shrink-0 opacity-80" aria-hidden />
                        <span className="truncate">{t(item.labelKey)}</span>
                      </button>
                    );
                  })}
                </div>
              ))}
            </nav>
          </aside>

          <div className="flex min-w-0 flex-1 flex-col">
            <header className="flex shrink-0 items-center border-b border-border/70 px-8 py-5 pr-14">
              <h2 className="font-display text-[length:var(--text-xl)] font-semibold tracking-tight text-foreground">
                {sectionTitle}
              </h2>
            </header>

            <div className="min-h-0 flex-1 overflow-y-auto px-8 py-7">
              {section === "language" ? (
                <section className="flex max-w-md flex-col gap-6">
                  <p className="text-[length:var(--text-sm)] leading-relaxed text-muted-foreground">
                    {t("settings.languageHint")}
                  </p>
                  <div className="flex flex-col gap-2">
                    <label
                      htmlFor="settings-locale"
                      className="text-[length:var(--text-sm)] font-medium text-foreground"
                    >
                      {t("settings.language")}
                    </label>
                    <Select
                      value={draftLocale}
                      onValueChange={(value) => {
                        setDraftLocale(value as AppLocale);
                        setLocaleSavedMessage("");
                      }}
                    >
                      <SelectTrigger id="settings-locale" className="w-full max-w-xs">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {locales.map((code) => (
                          <SelectItem key={code} value={code}>
                            {localeLabels[code] ?? code}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="flex flex-wrap items-center gap-3">
                    <Button
                      type="button"
                      disabled={!localeDirty}
                      onClick={() => {
                        setLocale(draftLocale);
                        setLocaleSavedMessage(t("settings.saved"));
                      }}
                    >
                      {t("settings.save")}
                    </Button>
                    {localeSavedMessage ? (
                      <span className="text-[length:var(--text-sm)] text-emerald-600 dark:text-emerald-400">
                        {localeSavedMessage}
                      </span>
                    ) : null}
                  </div>
                </section>
              ) : (
                <section className="flex max-w-lg flex-col gap-8">
                  <p className="text-[length:var(--text-sm)] leading-relaxed text-muted-foreground">
                    {t("settings.youtubeDescription")}
                  </p>

                  <div className="flex flex-col gap-2">
                    <label
                      htmlFor="settings-youtube-key"
                      className="text-[length:var(--text-sm)] font-medium text-foreground"
                    >
                      {t("settings.apiKey")}
                    </label>
                    <Input
                      id="settings-youtube-key"
                      type="password"
                      autoComplete="off"
                      disabled={loading || saving}
                      value={apiKey}
                      onChange={(event) => setApiKey(event.target.value)}
                      placeholder={t("settings.apiKeyPlaceholder")}
                      className="max-w-md"
                    />
                  </div>

                  <div className="flex flex-wrap items-center gap-3">
                    <Button
                      type="button"
                      disabled={loading || saving || !youtubeDirty}
                      onClick={() => void save()}
                    >
                      {saving ? t("settings.saving") : t("settings.save")}
                    </Button>
                    {savedMessage ? (
                      <span className="text-[length:var(--text-sm)] text-emerald-600 dark:text-emerald-400">
                        {t("settings.saved")}
                      </span>
                    ) : null}
                    {!loading && configured ? (
                      <span className="text-[length:var(--text-sm)] text-muted-foreground">
                        {t("settings.configured")}
                      </span>
                    ) : null}
                  </div>

                  {error ? (
                    <p className="text-[length:var(--text-sm)] text-red-500">{error}</p>
                  ) : null}

                  <div className="border-t border-border/70 pt-6">
                    <p className="text-[length:var(--text-sm)] leading-relaxed text-muted-foreground">
                      {t("settings.afterConfig")}{" "}
                      <button
                        type="button"
                        className="cursor-pointer font-medium text-primary underline-offset-4 transition-colors duration-150 ease-[cubic-bezier(0.23,1,0.32,1)] [@media(hover:hover)_and_(pointer:fine)]:hover:underline"
                        onClick={() => requestExit("crawler")}
                      >
                        {t("settings.crawlerLink")}
                      </button>{" "}
                      {t("settings.afterConfigSuffix")}
                    </p>
                  </div>
                </section>
              )}
            </div>
          </div>

          {confirmExit ? (
            <div
              className="absolute inset-0 z-30 flex items-center justify-center bg-black/50 p-6"
              role="presentation"
            >
              <div
                role="alertdialog"
                aria-modal="true"
                aria-labelledby="settings-unsaved-title"
                aria-describedby="settings-unsaved-desc"
                className="w-full max-w-sm rounded-[var(--radius-xl)] border border-border bg-card p-6 shadow-[var(--glass-shadow)]"
              >
                <h3
                  id="settings-unsaved-title"
                  className="font-display text-[length:var(--text-lg)] font-semibold tracking-tight"
                >
                  {t("settings.unsavedTitle")}
                </h3>
                <p
                  id="settings-unsaved-desc"
                  className="mt-2 text-[length:var(--text-sm)] leading-relaxed text-muted-foreground"
                >
                  {t("settings.unsavedDescription")}
                </p>
                <div className="mt-6 flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
                  <Button
                    type="button"
                    variant="ghost"
                    onClick={() => {
                      setConfirmExit(false);
                      setExitPendingAction(null);
                    }}
                  >
                    {t("settings.unsavedCancel")}
                  </Button>
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => {
                      discardAll();
                      finishExit(exitPendingAction ?? "close");
                    }}
                  >
                    {t("settings.unsavedDiscard")}
                  </Button>
                  <Button
                    type="button"
                    disabled={saving}
                    onClick={() => {
                      void (async () => {
                        const ok = await saveAll();
                        if (ok) {
                          finishExit(exitPendingAction ?? "close");
                        }
                      })();
                    }}
                  >
                    {saving ? t("settings.saving") : t("settings.unsavedSave")}
                  </Button>
                </div>
              </div>
            </div>
          ) : null}
        </div>
      </DialogContent>
    </Dialog>
  );
}

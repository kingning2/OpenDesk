/**
 * App settings page — language + YouTube API key persistence.
 *
 * @author Xiaoman
 * @created 2026-07-20
 */

import { Link } from "react-router";
import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Input,
  PageScaffold,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@desk/ui";
import { useI18n } from "../../i18n";
import type { AppLocale } from "../../i18n";
import { useYoutubeApiKeySettings } from "./use-youtube-api-key-settings";

/**
 * 应用设置页。
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @returns 设置页节点
 */
export function SettingsPage() {
  const { t, locale, setLocale, locales, localeLabels } = useI18n();
  const { apiKey, setApiKey, loading, saving, savedMessage, error, save, configured } =
    useYoutubeApiKeySettings();

  return (
    <PageScaffold subtitle={t("settings.subtitle")}>
      <div className="flex w-full max-w-xl flex-col gap-4">
        <Card variant="glass" className="w-full">
          <CardHeader>
            <CardTitle>{t("settings.language")}</CardTitle>
            <CardDescription>{t("settings.languageHint")}</CardDescription>
          </CardHeader>
          <CardContent>
            <Select
              value={locale}
              onValueChange={(value) => setLocale(value as AppLocale)}
            >
              <SelectTrigger className="w-full max-w-xs">
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
          </CardContent>
        </Card>

        <Card variant="glass" className="w-full">
          <CardHeader>
            <CardTitle>{t("settings.youtubeTitle")}</CardTitle>
            <CardDescription>{t("settings.youtubeDescription")}</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <label className="block space-y-1.5">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">
                {t("settings.apiKey")}
              </span>
              <Input
                type="password"
                autoComplete="off"
                disabled={loading || saving}
                value={apiKey}
                onChange={(event) => setApiKey(event.target.value)}
                placeholder={t("settings.apiKeyPlaceholder")}
              />
            </label>

            <div className="flex flex-wrap items-center gap-3">
              <Button type="button" disabled={loading || saving} onClick={() => void save()}>
                {saving ? t("settings.saving") : t("settings.save")}
              </Button>
              {savedMessage ? (
                <span className="text-[length:var(--text-sm)] text-emerald-600 dark:text-emerald-400">
                  {savedMessage}
                </span>
              ) : null}
              {!loading && configured ? (
                <span className="text-[length:var(--text-sm)] text-muted-foreground">
                  {t("settings.configured")}
                </span>
              ) : null}
            </div>

            {error ? <p className="text-[length:var(--text-sm)] text-red-500">{error}</p> : null}

            <p className="text-[length:var(--text-sm)] text-muted-foreground">
              {t("settings.afterConfig")}{" "}
              <Link
                to="/features/crawler"
                className="text-primary underline-offset-4 hover:underline"
              >
                {t("settings.crawlerLink")}
              </Link>{" "}
              {t("settings.afterConfigSuffix")}
            </p>
          </CardContent>
        </Card>
      </div>
    </PageScaffold>
  );
}

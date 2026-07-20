/**
 * App settings page — currently YouTube API key persistence.
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
} from "@desk/ui";
import { useYoutubeApiKeySettings } from "./use-youtube-api-key-settings";

export function SettingsPage() {
  const { apiKey, setApiKey, loading, saving, savedMessage, error, save, configured } =
    useYoutubeApiKeySettings();

  return (
    <PageScaffold subtitle="应用设置">
      <Card variant="glass" className="w-full max-w-xl">
        <CardHeader>
          <CardTitle>YouTube 采集</CardTitle>
          <CardDescription>
            配置 YouTube Data API v3 密钥。密钥保存在本机数据库，不会写入日志。
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <label className="block space-y-1.5">
            <span className="text-[length:var(--text-sm)] text-muted-foreground">API 密钥</span>
            <Input
              type="password"
              autoComplete="off"
              disabled={loading || saving}
              value={apiKey}
              onChange={(event) => setApiKey(event.target.value)}
              placeholder="粘贴 YouTube API 密钥"
            />
          </label>

          <div className="flex flex-wrap items-center gap-3">
            <Button type="button" disabled={loading || saving} onClick={() => void save()}>
              {saving ? "保存中…" : "保存"}
            </Button>
            {savedMessage ? (
              <span className="text-[length:var(--text-sm)] text-emerald-600 dark:text-emerald-400">
                {savedMessage}
              </span>
            ) : null}
            {!loading && configured ? (
              <span className="text-[length:var(--text-sm)] text-muted-foreground">已配置</span>
            ) : null}
          </div>

          {error ? <p className="text-[length:var(--text-sm)] text-red-500">{error}</p> : null}

          <p className="text-[length:var(--text-sm)] text-muted-foreground">
            配置完成后，可在{" "}
            <Link to="/features/crawler" className="text-primary underline-offset-4 hover:underline">
              YouTube 频道采集
            </Link>{" "}
            页面开始任务。
          </p>
        </CardContent>
      </Card>
    </PageScaffold>
  );
}

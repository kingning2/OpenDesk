/**
 * YouTube crawler page — API Key + operational progress (not phase logs).
 *
 * @author Xiaoman
 * @created 2026-07-16
 */

import { Card, CardContent, CardHeader, CardTitle, PageScaffold } from "@desk/ui";
import { useCrawlerJob } from "./use-crawler-job";

export function CrawlerPage() {
  const {
    apiKey,
    setApiKey,
    keywords,
    setKeywords,
    status,
    statusText,
    stopReason,
    message,
    currentKeyword,
    keywordAccepted,
    keywordScanned,
    acceptedCount,
    scannedCount,
    quotaUsed,
    keywordStats,
    busy,
    error,
    start,
    cancel,
  } = useCrawlerJob();

  const isQuotaStop = stopReason === "quota_exceeded";
  const isFailed = status === "failed";

  return (
    <PageScaffold subtitle="YouTube 频道采集 · 进度与配额自动停">
      <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden p-1">
        <Card variant="glass" className="w-full shrink-0">
          <CardHeader>
            <CardTitle>Crawler</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <label className="block space-y-1 text-[length:var(--text-sm)]">
              <span className="text-muted-foreground">YouTube API Key</span>
              <input
                type="password"
                autoComplete="off"
                value={apiKey}
                onChange={(event) => setApiKey(event.target.value)}
                placeholder="从 Google Cloud 控制台粘贴"
                className="w-full rounded-[var(--radius-md)] border border-border bg-background px-3 py-2"
              />
            </label>
            <label className="block space-y-1 text-[length:var(--text-sm)]">
              <span className="text-muted-foreground">关键词（逗号分隔）</span>
              <input
                type="text"
                value={keywords}
                onChange={(event) => setKeywords(event.target.value)}
                className="w-full rounded-[var(--radius-md)] border border-border bg-background px-3 py-2"
              />
            </label>
            <div className="flex flex-wrap items-center gap-2">
              <button
                type="button"
                disabled={busy || !apiKey.trim() || !keywords.trim()}
                onClick={() => void start()}
                className="rounded-[var(--radius-md)] bg-primary px-4 py-2 text-[length:var(--text-sm)] text-primary-foreground disabled:opacity-60"
              >
                {busy ? "爬取中…" : "开始爬取"}
              </button>
              <button
                type="button"
                disabled={!busy}
                onClick={() => void cancel()}
                className="rounded-[var(--radius-md)] border border-border px-4 py-2 text-[length:var(--text-sm)] disabled:opacity-60"
              >
                停止
              </button>
              <span className="text-[length:var(--text-sm)] text-muted-foreground">
                状态：{statusText}
              </span>
            </div>
            {error ? <p className="text-[length:var(--text-sm)] text-red-500">{error}</p> : null}
          </CardContent>
        </Card>

        <Card variant="glass" className="w-full shrink-0">
          <CardHeader>
            <CardTitle>当前进度</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 text-[length:var(--text-sm)]">
            {isQuotaStop ? (
              <p className="rounded-[var(--radius-md)] border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-amber-700 dark:text-amber-300">
                YouTube 配额已用尽，爬虫已自动停止。
              </p>
            ) : null}
            {isFailed && !isQuotaStop ? (
              <p className="rounded-[var(--radius-md)] border border-red-500/40 bg-red-500/10 px-3 py-2 text-red-600 dark:text-red-300">
                任务失败{error ? `：${error}` : ""}
              </p>
            ) : null}

            <p className="text-foreground">{message || "启动后将显示当前关键词与收录数量。"}</p>

            <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
              <div className="rounded-[var(--radius-md)] border border-border px-3 py-2">
                <div className="text-muted-foreground">当前关键词</div>
                <div className="mt-1 text-[length:var(--text-base)] font-medium">
                  {currentKeyword || "—"}
                </div>
              </div>
              <div className="rounded-[var(--radius-md)] border border-border px-3 py-2">
                <div className="text-muted-foreground">本词收录 / 扫描</div>
                <div className="mt-1 text-[length:var(--text-base)] font-medium">
                  {keywordAccepted} / {keywordScanned}
                </div>
              </div>
              <div className="rounded-[var(--radius-md)] border border-border px-3 py-2">
                <div className="text-muted-foreground">合计收录 / 扫描</div>
                <div className="mt-1 text-[length:var(--text-base)] font-medium">
                  {acceptedCount} / {scannedCount}
                </div>
              </div>
              <div className="rounded-[var(--radius-md)] border border-border px-3 py-2">
                <div className="text-muted-foreground">已用配额（估算）</div>
                <div className="mt-1 text-[length:var(--text-base)] font-medium">{quotaUsed}</div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card variant="glass" className="flex min-h-0 flex-1 flex-col overflow-hidden">
          <CardHeader className="shrink-0">
            <CardTitle>各关键词结果</CardTitle>
          </CardHeader>
          <CardContent className="min-h-0 flex-1 overflow-auto">
            {keywordStats.length === 0 ? (
              <p className="text-[length:var(--text-sm)] text-muted-foreground">尚无关键词进度</p>
            ) : (
              <table className="w-full text-left text-[length:var(--text-sm)]">
                <thead className="text-muted-foreground">
                  <tr>
                    <th className="pb-2 pr-4 font-medium">关键词</th>
                    <th className="pb-2 pr-4 font-medium">扫描</th>
                    <th className="pb-2 font-medium">收录</th>
                  </tr>
                </thead>
                <tbody>
                  {keywordStats.map((row) => (
                    <tr key={row.keyword} className="border-t border-border">
                      <td className="py-2 pr-4">
                        {row.keyword}
                        {row.keyword === currentKeyword && busy ? (
                          <span className="ml-2 text-muted-foreground">（进行中）</span>
                        ) : null}
                      </td>
                      <td className="py-2 pr-4">{row.scanned}</td>
                      <td className="py-2">{row.accepted}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </CardContent>
        </Card>
      </div>
    </PageScaffold>
  );
}

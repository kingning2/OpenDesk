/**
 * YouTube crawler page — API Key from UI, process log panel.
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
    jobId,
    status,
    summary,
    logs,
    busy,
    error,
    start,
    cancel,
  } = useCrawlerJob();

  return (
    <PageScaffold subtitle="YouTube channel crawl — API Key from desktop UI">
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
              <span className="text-muted-foreground">Keywords（逗号分隔）</span>
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
                {busy ? "Running..." : "Start crawl"}
              </button>
              <button
                type="button"
                disabled={!jobId || !busy}
                onClick={() => void cancel()}
                className="rounded-[var(--radius-md)] border border-border px-4 py-2 text-[length:var(--text-sm)] disabled:opacity-60"
              >
                Cancel
              </button>
              <span className="text-[length:var(--text-sm)] text-muted-foreground">
                status={status}
                {jobId ? ` · job=${jobId.slice(0, 8)}…` : ""}
              </span>
            </div>
            {summary ? (
              <p className="text-[length:var(--text-sm)] text-muted-foreground">{summary}</p>
            ) : null}
            {error ? <p className="text-[length:var(--text-sm)] text-destructive">{error}</p> : null}
          </CardContent>
        </Card>

        <Card variant="glass" className="flex min-h-0 flex-1 flex-col overflow-hidden">
          <CardHeader className="shrink-0">
            <CardTitle>Process logs</CardTitle>
          </CardHeader>
          <CardContent className="min-h-0 flex-1 overflow-auto">
            <pre className="whitespace-pre-wrap font-mono text-[length:var(--text-xs)] leading-relaxed text-foreground">
              {logs.length === 0
                ? "等待任务日志…"
                : logs
                    .map((line) => {
                      const seq = line.seq ?? "-";
                      const phase = line.phase ?? "-";
                      const message = line.message ?? "";
                      return `[${seq}] ${phase}  ${message}`;
                    })
                    .join("\n")}
            </pre>
          </CardContent>
        </Card>
      </div>
    </PageScaffold>
  );
}

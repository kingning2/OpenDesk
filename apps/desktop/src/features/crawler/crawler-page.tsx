/**
 * YouTube crawler page — flow monitor with stage-linked detail panel.
 */

import { Link } from "react-router";
import { useMemo, useRef, useState } from "react";
import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  PageScaffold,
  ScrollArea,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  useTheme,
} from "@desk/ui";
import {
  Background,
  ColorMode,
  Handle,
  MarkerType,
  type Edge,
  type Node,
  type NodeProps,
  Position,
  ReactFlow,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import type { KeywordBatchRow } from "@desk/platform/ipc/crawler";
import { useT, useI18n } from "../../i18n";
import {
  useCrawlerJob,
  type ChannelResultRow,
  type CrawlerLogRow,
  type KeywordStatRow,
} from "./use-crawler-job";

type FlowStage = "source" | "search" | "summary";
type PanelView = "process" | "results";

interface MonitorNodeData extends Record<string, unknown> {
  title: string;
  subtitle: string;
  value: string;
  tone: "idle" | "running" | "done" | "warn" | "error";
  selected?: boolean;
}

function MonitorNode({ data }: NodeProps<Node<MonitorNodeData>>) {
  let toneClass = "border-border bg-card/80";
  switch (data.tone) {
    case "running":
      toneClass = "border-sky-500/40 bg-sky-500/10";
      break;
    case "done":
      toneClass = "border-emerald-500/40 bg-emerald-500/10";
      break;
    case "warn":
      toneClass = "border-amber-500/40 bg-amber-500/10";
      break;
    case "error":
      toneClass = "border-red-500/40 bg-red-500/10";
      break;
    default:
      break;
  }

  return (
    <div
      className={`w-[220px] cursor-pointer rounded-[var(--radius-lg)] border px-4 py-3 shadow-sm transition-[box-shadow,border-color] ${toneClass} ${
        data.selected ? "ring-2 ring-primary/60 ring-offset-2 ring-offset-background" : ""
      }`}
    >
      <Handle
        type="target"
        position={Position.Left}
        className="!size-2.5 !border-2 !border-background !bg-primary"
      />
      <div className="text-xs text-muted-foreground">{data.subtitle}</div>
      <div className="mt-1 text-sm font-semibold">{data.title}</div>
      <div className="mt-2 text-[length:var(--text-sm)] leading-snug text-muted-foreground">{data.value}</div>
      <Handle
        type="source"
        position={Position.Right}
        className="!size-2.5 !border-2 !border-background !bg-primary"
      />
    </div>
  );
}

const nodeTypes = { monitor: MonitorNode };

type Translate = (key: string, params?: Record<string, string | number>) => string;

function stageLabel(stage: FlowStage, t: Translate): string {
  if (stage === "source") return t("crawler.stage.source");
  if (stage === "search") return t("crawler.stage.search");
  return t("crawler.stage.summary");
}

function batchLabel(row: KeywordBatchRow, index: number, t: Translate): string {
  return t("crawler.batchLabel", {
    index: index + 1,
    count: row.keyword_count.toLocaleString(),
  });
}

function formatLogTime(iso: string, locale: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return "";
  }
  return date.toLocaleTimeString(locale === "zh-CN" ? "zh-CN" : "en-US", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

function isSearchPhaseLog(row: CrawlerLogRow): boolean {
  const text = row.message.toLowerCase();
  return (
    text.includes("search.list") ||
    text.includes("channels.list") ||
    text.includes("playlistitems") ||
    text.includes("begin keyword") ||
    text.includes("keyword done") ||
    row.phase === "search" ||
    row.phase === "channel"
  );
}

function LogList({ rows, emptyText }: { rows: CrawlerLogRow[]; emptyText: string }) {
  const { locale } = useI18n();
  if (rows.length === 0) {
    return <p className="text-[length:var(--text-sm)] text-muted-foreground">{emptyText}</p>;
  }
  return (
    <div className="space-y-2">
      {rows.map((row) => (
        <div
          key={row.event_id}
          className="rounded-[var(--radius-md)] border border-border bg-card/30 px-3 py-2"
        >
          <div className="flex items-start justify-between gap-2">
            <p className="text-[length:var(--text-sm)] leading-relaxed">{row.message}</p>
            <span className="shrink-0 text-xs text-muted-foreground">
              {formatLogTime(row.occurred_at, locale)}
            </span>
          </div>
        </div>
      ))}
    </div>
  );
}

function ChannelList({ rows }: { rows: ChannelResultRow[] }) {
  const t = useT();
  if (rows.length === 0) {
    return (
      <p className="text-[length:var(--text-sm)] text-muted-foreground">{t("crawler.channelsEmpty")}</p>
    );
  }
  return (
    <div className="space-y-2">
      {rows.map((row) => (
        <div
          key={row.channel_id}
          className="rounded-[var(--radius-md)] border border-border bg-card/30 px-3 py-2"
        >
          <div className="text-[length:var(--text-sm)] font-medium">{row.title}</div>
          <div className="mt-1 flex flex-wrap gap-x-3 gap-y-1 text-xs text-muted-foreground">
            {row.subscriber_count != null ? (
              <span>{t("crawler.subscribers", { count: row.subscriber_count.toLocaleString() })}</span>
            ) : null}
            {row.email ? <span>{row.email}</span> : null}
            {row.keyword ? <span>{t("crawler.fromKeyword", { keyword: row.keyword })}</span> : null}
          </div>
        </div>
      ))}
    </div>
  );
}

function KeywordStatsList({ rows }: { rows: KeywordStatRow[] }) {
  const t = useT();
  if (rows.length === 0) {
    return (
      <p className="text-[length:var(--text-sm)] text-muted-foreground">{t("crawler.keywordStatsEmpty")}</p>
    );
  }
  return (
    <div className="space-y-2">
      {rows.map((row) => (
        <div
          key={row.keyword}
          className="flex items-center justify-between rounded-[var(--radius-md)] border border-border bg-card/30 px-3 py-2"
        >
          <span className="truncate text-[length:var(--text-sm)] font-medium">{row.keyword}</span>
          <span className="shrink-0 text-xs text-muted-foreground">
            {t("crawler.scannedAccepted", {
              scanned: row.scanned.toLocaleString(),
              accepted: row.accepted.toLocaleString(),
            })}
          </span>
        </div>
      ))}
    </div>
  );
}

function formatKeywordProgress(done: number, total: number, active: boolean): string {
  if (total <= 0) {
    return "—";
  }
  const position = Math.min(done + (active ? 1 : 0), total);
  return `${position.toLocaleString()} / ${total.toLocaleString()}`;
}

function KeywordProgressBanner({
  done,
  total,
  active,
  message,
}: {
  done: number;
  total: number;
  active: boolean;
  message?: string;
}) {
  const t = useT();
  if (total <= 0) {
    return null;
  }
  const position = Math.min(done + (active ? 1 : 0), total);
  const percent = Math.min(100, Math.round((position / total) * 100));
  return (
    <div className="mt-3 rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-3">
      <div className="flex items-center justify-between gap-3">
        <span className="text-[length:var(--text-sm)] text-muted-foreground">
          {t("crawler.batchKeywordProgress")}
        </span>
        <span className="text-[length:var(--text-sm)] font-semibold tabular-nums">
          {formatKeywordProgress(done, total, active)}
        </span>
      </div>
      <div className="mt-2 h-2 overflow-hidden rounded-full bg-muted/60">
        <div
          className="h-full rounded-full bg-primary transition-[width] duration-300"
          style={{ width: `${percent}%` }}
        />
      </div>
      {message ? (
        <p className="mt-2 text-xs text-muted-foreground">{message}</p>
      ) : null}
    </div>
  );
}

interface DetailPanelProps {
  stage: FlowStage;
  view: PanelView;
  selectedBatch: KeywordBatchRow | undefined;
  batches: KeywordBatchRow[];
  batchId: string;
  importMessage: string;
  statusText: string;
  currentKeyword: string;
  scannedCount: number;
  acceptedCount: number;
  logs: CrawlerLogRow[];
  keywordStats: KeywordStatRow[];
  channelResults: ChannelResultRow[];
  message: string;
  keywordsDone: number;
  keywordsTotal: number;
  busy: boolean;
}

function DetailPanel({
  stage,
  view,
  selectedBatch,
  batches,
  batchId,
  importMessage,
  statusText,
  currentKeyword,
  scannedCount,
  acceptedCount,
  logs,
  keywordStats,
  channelResults,
  message,
  keywordsDone,
  keywordsTotal,
  busy,
}: DetailPanelProps) {
  const t = useT();
  const searchLogs = useMemo(() => logs.filter(isSearchPhaseLog), [logs]);

  if (stage === "source") {
    if (view === "results") {
      return (
        <div className="space-y-3">
          <div className="rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-3">
            <div className="text-xs text-muted-foreground">{t("crawler.currentBatch")}</div>
            <div className="mt-1 text-[length:var(--text-sm)] font-medium">
              {selectedBatch
                ? t("crawler.keywordsReady", {
                    count: selectedBatch.keyword_count.toLocaleString(),
                  })
                : t("crawler.noBatchSelected")}
            </div>
            {batchId ? (
              <div className="mt-2 truncate text-xs text-muted-foreground">
                {t("crawler.batchId", { id: batchId })}
              </div>
            ) : null}
          </div>
          <div>
            <div className="mb-2 text-[length:var(--text-sm)] font-medium">{t("crawler.allBatches")}</div>
            {batches.length === 0 ? (
              <p className="text-[length:var(--text-sm)] text-muted-foreground">
                {t("crawler.batchesEmpty")}
              </p>
            ) : (
              <div className="space-y-2">
                {batches.map((row, index) => (
                  <div
                    key={row.batch_id}
                    className={`rounded-[var(--radius-md)] border px-3 py-2 text-[length:var(--text-sm)] ${
                      row.batch_id === batchId
                        ? "border-primary/40 bg-primary/10"
                        : "border-border bg-card/30"
                    }`}
                  >
                    {batchLabel(row, index, t)}
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      );
    }
    return (
      <div className="space-y-3">
        <div className="rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-3">
          <div className="text-xs text-muted-foreground">{t("crawler.importStatus")}</div>
          <div className="mt-1 text-[length:var(--text-sm)]">
            {importMessage || t("crawler.importHint")}
          </div>
        </div>
        <div>
          <div className="mb-2 text-[length:var(--text-sm)] font-medium">{t("crawler.prepLogs")}</div>
          <LogList
            rows={logs.filter((row) => row.phase === "import" || row.phase === "setup")}
            emptyText={t("crawler.prepLogsEmpty")}
          />
        </div>
      </div>
    );
  }

  if (stage === "search") {
    if (view === "results") {
      return (
        <div className="space-y-3">
          <KeywordProgressBanner
            done={keywordsDone}
            total={keywordsTotal}
            active={busy}
            message={
              currentKeyword
                ? t("crawler.currentKeywordLabel", { keyword: currentKeyword })
                : undefined
            }
          />
          <div className="grid grid-cols-2 gap-2">
            <div className="rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-2">
              <div className="text-xs text-muted-foreground">{t("crawler.currentKeyword")}</div>
              <div className="mt-1 truncate text-[length:var(--text-sm)] font-medium">
                {currentKeyword || "—"}
              </div>
            </div>
            <div className="rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-2">
              <div className="text-xs text-muted-foreground">{t("crawler.browseAccept")}</div>
              <div className="mt-1 text-[length:var(--text-sm)] font-medium">
                {scannedCount.toLocaleString()} / {acceptedCount.toLocaleString()}
              </div>
            </div>
          </div>
          <div>
            <div className="mb-2 text-[length:var(--text-sm)] font-medium">
              {t("crawler.keywordProgress")}
            </div>
            <KeywordStatsList rows={keywordStats} />
          </div>
        </div>
      );
    }
    return (
      <div className="space-y-3">
        <KeywordProgressBanner
          done={keywordsDone}
          total={keywordsTotal}
          active={busy}
          message={
            currentKeyword
              ? t("crawler.currentKeywordLabel", { keyword: currentKeyword })
              : undefined
          }
        />
        <div>
          <div className="mb-2 text-[length:var(--text-sm)] font-medium">{t("crawler.searchProcess")}</div>
          <LogList rows={searchLogs} emptyText={t("crawler.searchLogsEmpty")} />
        </div>
      </div>
    );
  }

  if (view === "results") {
    return <ChannelList rows={channelResults} />;
  }

  return (
    <div className="space-y-3">
      <div className="grid grid-cols-2 gap-2">
        <div className="rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-2">
          <div className="text-xs text-muted-foreground">{t("crawler.jobStatus")}</div>
          <div className="mt-1 text-[length:var(--text-sm)] font-medium">{statusText}</div>
        </div>
        <div className="rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-2">
          <div className="text-xs text-muted-foreground">{t("crawler.summary")}</div>
          <div className="mt-1 text-[length:var(--text-sm)] font-medium">
            {channelResults.length > 0
              ? t("crawler.channelsAccepted", {
                  count: channelResults.length.toLocaleString(),
                })
              : message || t("crawler.waitingComplete")}
          </div>
        </div>
      </div>
      <div>
        <div className="mb-2 text-[length:var(--text-sm)] font-medium">{t("crawler.fullProcess")}</div>
        <LogList rows={logs} emptyText={t("crawler.fullLogsEmpty")} />
      </div>
    </div>
  );
}

export function CrawlerPage() {
  const t = useT();
  const { resolvedTheme } = useTheme();
  const {
    apiKeyConfigured,
    apiKeyLoading,
    batchId,
    setBatchId,
    batches,
    selectedBatch,
    importMessage,
    importing,
    importCsvFile,
    status,
    statusText,
    stopReason,
    message,
    currentKeyword,
    acceptedCount,
    scannedCount,
    keywordsDone,
    keywordsTotal,
    keywordStats,
    logs,
    channelResults,
    busy,
    error,
    start,
    cancel,
  } = useCrawlerJob();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [manualStage, setManualStage] = useState<FlowStage | null>(null);
  const [panelView, setPanelView] = useState<PanelView>("results");

  const isQuotaStop = stopReason === "quota_exceeded";
  const isFailed = status === "failed";
  const canStart = Boolean(apiKeyConfigured && batchId && (selectedBatch?.keyword_count ?? 0) > 0);

  const selectedStage = useMemo<FlowStage>(() => {
    if (manualStage) {
      return manualStage;
    }
    if (busy) {
      return "search";
    }
    if (channelResults.length > 0) {
      return "summary";
    }
    return "source";
  }, [manualStage, busy, channelResults.length]);

  const nodes = useMemo<Node<MonitorNodeData>[]>(() => {
    const sourceTone: MonitorNodeData["tone"] = selectedBatch ? "done" : "idle";
    const searchTone: MonitorNodeData["tone"] = isFailed
      ? "error"
      : isQuotaStop
        ? "warn"
        : busy
          ? "running"
          : acceptedCount > 0 || scannedCount > 0
            ? "done"
            : "idle";
    const summaryTone: MonitorNodeData["tone"] = isFailed
      ? "error"
      : isQuotaStop
        ? "warn"
        : status === "completed"
          ? "done"
          : busy
            ? "running"
            : channelResults.length > 0
              ? "done"
              : "idle";

    const keywordCount = selectedBatch?.keyword_count.toLocaleString() ?? "0";

    return [
      {
        id: "source",
        type: "monitor",
        position: { x: 0, y: 80 },
        data: {
          title: t("crawler.stage.source"),
          subtitle: selectedBatch
            ? t("crawler.wordsPending", { count: keywordCount })
            : t("crawler.importKeywordsFirst"),
          value: selectedBatch ? t("crawler.ready") : t("crawler.importCsvToStart"),
          tone: sourceTone,
          selected: selectedStage === "source",
        },
      },
      {
        id: "search",
        type: "monitor",
        position: { x: 300, y: 80 },
        data: {
          title: busy ? t("crawler.searchingChannels") : t("crawler.stage.search"),
          subtitle:
            keywordsTotal > 0
              ? formatKeywordProgress(keywordsDone, keywordsTotal, busy)
              : currentKeyword || t("crawler.waitingStart"),
          value: t("crawler.browsedAccepted", {
            scanned: scannedCount.toLocaleString(),
            accepted: acceptedCount.toLocaleString(),
          }),
          tone: searchTone,
          selected: selectedStage === "search",
        },
      },
      {
        id: "summary",
        type: "monitor",
        position: { x: 600, y: 80 },
        data: {
          title: t("crawler.stage.summary"),
          subtitle: statusText,
          value:
            channelResults.length > 0
              ? t("crawler.channelsFound", {
                  count: channelResults.length.toLocaleString(),
                })
              : message || t("crawler.showWhenDone"),
          tone: summaryTone,
          selected: selectedStage === "summary",
        },
      },
    ];
  }, [
    acceptedCount,
    busy,
    channelResults.length,
    currentKeyword,
    isFailed,
    isQuotaStop,
    message,
    scannedCount,
    selectedBatch,
    keywordsDone,
    keywordsTotal,
    selectedStage,
    status,
    statusText,
    t,
  ]);

  const edges = useMemo<Edge[]>(
    () => [
      {
        id: "source-search",
        source: "source",
        target: "search",
        type: "smoothstep",
        animated: busy,
        style: { stroke: "var(--color-primary)", strokeWidth: 2 },
        markerEnd: { type: MarkerType.ArrowClosed, color: "var(--color-primary)" },
      },
      {
        id: "search-summary",
        source: "search",
        target: "summary",
        type: "smoothstep",
        animated: busy,
        style: { stroke: "var(--color-primary)", strokeWidth: 2 },
        markerEnd: { type: MarkerType.ArrowClosed, color: "var(--color-primary)" },
      },
    ],
    [busy],
  );

  function handleStageSelect(stage: FlowStage) {
    setManualStage(stage);
    if (stage === "summary") {
      setPanelView("results");
    } else if (stage === "search") {
      setPanelView(busy ? "process" : "results");
    } else {
      setPanelView("results");
    }
  }

  function handleStart() {
    setManualStage(null);
    setPanelView("process");
    void start();
  }

  return (
    <PageScaffold fill containerWidth="full" containerPadding="sm" subtitle={t("crawler.subtitle")}>
      <div className="flex min-h-0 flex-1 flex-col gap-3">
        <Card variant="glass" padding="sm" className="shrink-0">
          <div className="grid gap-3 lg:grid-cols-[auto_minmax(180px,0.7fr)_auto] lg:items-end">
            <div className="flex items-end gap-2">
              <input
                ref={fileInputRef}
                type="file"
                accept=".csv,text/csv"
                className="hidden"
                disabled={importing || busy}
                onChange={(event) => {
                  const file = event.target.files?.[0];
                  if (file) {
                    void importCsvFile(file);
                  }
                  event.target.value = "";
                }}
              />
              <Button
                type="button"
                variant="outline"
                disabled={importing || busy}
                onClick={() => fileInputRef.current?.click()}
              >
                {importing ? t("crawler.importing") : t("crawler.importKeywords")}
              </Button>
            </div>

            <label className="block min-w-[180px] space-y-1.5">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">
                {t("crawler.keywordBatch")}
              </span>
              {batches.length === 0 ? (
                <Select disabled>
                  <SelectTrigger>
                    <SelectValue placeholder={t("crawler.importKeywordsFirst")} />
                  </SelectTrigger>
                </Select>
              ) : (
                <Select value={batchId} onValueChange={setBatchId} disabled={busy}>
                  <SelectTrigger>
                    <SelectValue placeholder={t("crawler.selectBatch")} />
                  </SelectTrigger>
                  <SelectContent>
                    {batches.map((row, index) => (
                      <SelectItem key={row.batch_id} value={row.batch_id}>
                        {batchLabel(row, index, t)}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              )}
            </label>

            <div className="flex items-end gap-2 lg:justify-end">
              <Button type="button" disabled={busy || !canStart} onClick={handleStart}>
                {busy ? t("crawler.crawling") : t("crawler.startCrawl")}
              </Button>
              <Button type="button" variant="outline" disabled={!busy} onClick={() => void cancel()}>
                {t("crawler.stop")}
              </Button>
            </div>
          </div>

          {importMessage ? (
            <p className="mt-3 text-[length:var(--text-sm)] text-muted-foreground">{importMessage}</p>
          ) : null}
          {!apiKeyLoading && !apiKeyConfigured ? (
            <p className="mt-3 rounded-[var(--radius-md)] border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-[length:var(--text-sm)] text-amber-700 dark:text-amber-300">
              {t("crawler.needApiKeyPrefix")}{" "}
              <Link to="/settings" className="font-medium underline underline-offset-4">
                {t("crawler.settingsLink")}
              </Link>{" "}
              {t("crawler.needApiKeySuffix")}
            </p>
          ) : null}
          {isQuotaStop ? (
            <p className="mt-3 rounded-[var(--radius-md)] border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-[length:var(--text-sm)] text-amber-700 dark:text-amber-300">
              {t("crawler.quotaPaused")}
            </p>
          ) : null}
          {isFailed ? (
            <p className="mt-3 rounded-[var(--radius-md)] border border-red-500/40 bg-red-500/10 px-3 py-2 text-[length:var(--text-sm)] text-red-600 dark:text-red-300">
              {error
                ? t("crawler.crawlFailedWithError", { error })
                : t("crawler.crawlFailed")}
            </p>
          ) : null}
          {!isFailed && error ? (
            <p className="mt-3 text-[length:var(--text-sm)] text-red-500">{error}</p>
          ) : null}
          <KeywordProgressBanner
            done={keywordsDone}
            total={keywordsTotal}
            active={busy}
            message={message}
          />
        </Card>

        <div className="grid min-h-0 flex-1 gap-3 xl:grid-cols-[1.45fr_1fr]">
          <Card variant="glass" padding="none" className="flex min-h-0 flex-col overflow-hidden">
            <CardHeader compact className="shrink-0">
              <CardTitle>{t("crawler.flowTitle")}</CardTitle>
              <CardDescription>{t("crawler.flowDescription")}</CardDescription>
            </CardHeader>
            <CardContent padding="none" className="min-h-0 flex-1">
              <div className="h-full min-h-[280px] w-full">
                <ReactFlow
                  fitView
                  fitViewOptions={{ padding: 0.35, minZoom: 0.85, maxZoom: 1 }}
                  colorMode={(resolvedTheme === "dark" ? "dark" : "light") as ColorMode}
                  nodes={nodes}
                  edges={edges}
                  nodeTypes={nodeTypes}
                  nodesDraggable={false}
                  nodesConnectable={false}
                  elementsSelectable
                  selectNodesOnDrag={false}
                  panOnDrag={false}
                  zoomOnScroll={false}
                  preventScrolling
                  proOptions={{ hideAttribution: true }}
                  className="bg-transparent"
                  onNodeClick={(_event, node) => handleStageSelect(node.id as FlowStage)}
                >
                  <Background gap={18} color="var(--color-border)" />
                </ReactFlow>
              </div>
            </CardContent>
          </Card>

          <Card variant="glass" padding="none" className="flex min-h-0 flex-col overflow-hidden">
            <CardHeader compact className="shrink-0 space-y-3">
              <div className="flex items-start justify-between gap-3">
                <div>
                  <CardTitle>{stageLabel(selectedStage, t)}</CardTitle>
                  <CardDescription>
                    {panelView === "process"
                      ? t("crawler.panelProcessDesc")
                      : t("crawler.panelResultsDesc")}
                  </CardDescription>
                </div>
                <div className="flex shrink-0 gap-1 rounded-[var(--radius-md)] border border-border bg-card/40 p-0.5">
                  <Button
                    type="button"
                    size="sm"
                    variant={panelView === "process" ? "default" : "ghost"}
                    onClick={() => setPanelView("process")}
                  >
                    {t("crawler.process")}
                  </Button>
                  <Button
                    type="button"
                    size="sm"
                    variant={panelView === "results" ? "default" : "ghost"}
                    onClick={() => setPanelView("results")}
                  >
                    {t("crawler.results")}
                  </Button>
                </div>
              </div>
            </CardHeader>

            <ScrollArea className="min-h-0 flex-1 p-[10px]">
              <DetailPanel
                stage={selectedStage}
                view={panelView}
                selectedBatch={selectedBatch}
                batches={batches}
                batchId={batchId}
                importMessage={importMessage}
                statusText={statusText}
                currentKeyword={currentKeyword}
                scannedCount={scannedCount}
                acceptedCount={acceptedCount}
                logs={logs}
                keywordStats={keywordStats}
                channelResults={channelResults}
                message={message}
                keywordsDone={keywordsDone}
                keywordsTotal={keywordsTotal}
                busy={busy}
              />
            </ScrollArea>
          </Card>
        </div>
      </div>
    </PageScaffold>
  );
}

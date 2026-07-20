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

const STAGE_LABELS: Record<FlowStage, string> = {
  source: "关键词库",
  search: "频道搜索",
  summary: "采集结果",
};

function batchLabel(row: KeywordBatchRow, index: number): string {
  return `批次 ${index + 1} · ${row.keyword_count.toLocaleString()} 个关键词`;
}

function formatLogTime(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return "";
  }
  return date.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit", second: "2-digit" });
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
            <span className="shrink-0 text-xs text-muted-foreground">{formatLogTime(row.occurred_at)}</span>
          </div>
        </div>
      ))}
    </div>
  );
}

function ChannelList({ rows }: { rows: ChannelResultRow[] }) {
  if (rows.length === 0) {
    return (
      <p className="text-[length:var(--text-sm)] text-muted-foreground">符合条件的频道会显示在这里。</p>
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
              <span>订阅 {row.subscriber_count.toLocaleString()}</span>
            ) : null}
            {row.email ? <span>{row.email}</span> : null}
            {row.keyword ? <span>来自「{row.keyword}」</span> : null}
          </div>
        </div>
      ))}
    </div>
  );
}

function KeywordStatsList({ rows }: { rows: KeywordStatRow[] }) {
  if (rows.length === 0) {
    return <p className="text-[length:var(--text-sm)] text-muted-foreground">开始采集后会按关键词汇总进度。</p>;
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
            浏览 {row.scanned.toLocaleString()} · 收录 {row.accepted.toLocaleString()}
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
  if (total <= 0) {
    return null;
  }
  const position = Math.min(done + (active ? 1 : 0), total);
  const percent = Math.min(100, Math.round((position / total) * 100));
  return (
    <div className="mt-3 rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-3">
      <div className="flex items-center justify-between gap-3">
        <span className="text-[length:var(--text-sm)] text-muted-foreground">批次关键词进度</span>
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
  const searchLogs = useMemo(() => logs.filter(isSearchPhaseLog), [logs]);

  if (stage === "source") {
    if (view === "results") {
      return (
        <div className="space-y-3">
          <div className="rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-3">
            <div className="text-xs text-muted-foreground">当前批次</div>
            <div className="mt-1 text-[length:var(--text-sm)] font-medium">
              {selectedBatch
                ? `${selectedBatch.keyword_count.toLocaleString()} 个关键词已就绪`
                : "尚未选择批次"}
            </div>
            {batchId ? (
              <div className="mt-2 truncate text-xs text-muted-foreground">批次 ID：{batchId}</div>
            ) : null}
          </div>
          <div>
            <div className="mb-2 text-[length:var(--text-sm)] font-medium">全部批次</div>
            {batches.length === 0 ? (
              <p className="text-[length:var(--text-sm)] text-muted-foreground">导入 CSV 后会出现在这里。</p>
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
                    {batchLabel(row, index)}
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
          <div className="text-xs text-muted-foreground">导入状态</div>
          <div className="mt-1 text-[length:var(--text-sm)]">
            {importMessage || "点击顶部「导入关键词」上传 CSV 文件。"}
          </div>
        </div>
        <div>
          <div className="mb-2 text-[length:var(--text-sm)] font-medium">准备记录</div>
          <LogList rows={logs.filter((row) => row.phase === "import" || row.phase === "setup")} emptyText="暂无准备阶段记录。" />
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
            message={currentKeyword ? `当前关键词：${currentKeyword}` : undefined}
          />
          <div className="grid grid-cols-2 gap-2">
            <div className="rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-2">
              <div className="text-xs text-muted-foreground">当前关键词</div>
              <div className="mt-1 truncate text-[length:var(--text-sm)] font-medium">
                {currentKeyword || "—"}
              </div>
            </div>
            <div className="rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-2">
              <div className="text-xs text-muted-foreground">浏览 / 收录</div>
              <div className="mt-1 text-[length:var(--text-sm)] font-medium">
                {scannedCount.toLocaleString()} / {acceptedCount.toLocaleString()}
              </div>
            </div>
          </div>
          <div>
            <div className="mb-2 text-[length:var(--text-sm)] font-medium">关键词进度</div>
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
          message={currentKeyword ? `当前关键词：${currentKeyword}` : undefined}
        />
        <div>
          <div className="mb-2 text-[length:var(--text-sm)] font-medium">搜索与拉取过程</div>
          <LogList rows={searchLogs} emptyText="开始采集后，API 调用与关键词切换会显示在这里。" />
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
          <div className="text-xs text-muted-foreground">任务状态</div>
          <div className="mt-1 text-[length:var(--text-sm)] font-medium">{statusText}</div>
        </div>
        <div className="rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-2">
          <div className="text-xs text-muted-foreground">摘要</div>
          <div className="mt-1 text-[length:var(--text-sm)] font-medium">
            {channelResults.length > 0
              ? `已收录 ${channelResults.length.toLocaleString()} 个频道`
              : message || "等待采集完成"}
          </div>
        </div>
      </div>
      <div>
        <div className="mb-2 text-[length:var(--text-sm)] font-medium">完整过程</div>
        <LogList rows={logs} emptyText="暂无动态，开始采集后会显示在这里。" />
      </div>
    </div>
  );
}

export function CrawlerPage() {
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
          title: "关键词库",
          subtitle: selectedBatch ? `${keywordCount} 个词待采集` : "请先导入关键词",
          value: selectedBatch ? "已准备就绪" : "导入 CSV 后即可开始",
          tone: sourceTone,
          selected: selectedStage === "source",
        },
      },
      {
        id: "search",
        type: "monitor",
        position: { x: 300, y: 80 },
        data: {
          title: busy ? "正在搜索频道" : "频道搜索",
          subtitle:
            keywordsTotal > 0
              ? formatKeywordProgress(keywordsDone, keywordsTotal, busy)
              : currentKeyword || "等待开始",
          value: `已浏览 ${scannedCount.toLocaleString()} · 已收录 ${acceptedCount.toLocaleString()}`,
          tone: searchTone,
          selected: selectedStage === "search",
        },
      },
      {
        id: "summary",
        type: "monitor",
        position: { x: 600, y: 80 },
        data: {
          title: "采集结果",
          subtitle: statusText,
          value:
            channelResults.length > 0
              ? `已找到 ${channelResults.length.toLocaleString()} 个频道`
              : message || "完成后会显示在这里",
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
    <PageScaffold fill containerWidth="full" containerPadding="sm" subtitle="YouTube 频道采集">
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
                {importing ? "导入中…" : "导入关键词"}
              </Button>
            </div>

            <label className="block min-w-[180px] space-y-1.5">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">关键词批次</span>
              {batches.length === 0 ? (
                <Select disabled>
                  <SelectTrigger>
                    <SelectValue placeholder="请先导入关键词" />
                  </SelectTrigger>
                </Select>
              ) : (
                <Select value={batchId} onValueChange={setBatchId} disabled={busy}>
                  <SelectTrigger>
                    <SelectValue placeholder="选择批次" />
                  </SelectTrigger>
                  <SelectContent>
                    {batches.map((row, index) => (
                      <SelectItem key={row.batch_id} value={row.batch_id}>
                        {batchLabel(row, index)}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              )}
            </label>

            <div className="flex items-end gap-2 lg:justify-end">
              <Button type="button" disabled={busy || !canStart} onClick={handleStart}>
                {busy ? "采集中…" : "开始采集"}
              </Button>
              <Button type="button" variant="outline" disabled={!busy} onClick={() => void cancel()}>
                停止
              </Button>
            </div>
          </div>

          {importMessage ? (
            <p className="mt-3 text-[length:var(--text-sm)] text-muted-foreground">{importMessage}</p>
          ) : null}
          {!apiKeyLoading && !apiKeyConfigured ? (
            <p className="mt-3 rounded-[var(--radius-md)] border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-[length:var(--text-sm)] text-amber-700 dark:text-amber-300">
              请先在{" "}
              <Link to="/settings" className="font-medium underline underline-offset-4">
                设置
              </Link>{" "}
              中配置 YouTube API 密钥，再开始采集。
            </p>
          ) : null}
          {isQuotaStop ? (
            <p className="mt-3 rounded-[var(--radius-md)] border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-[length:var(--text-sm)] text-amber-700 dark:text-amber-300">
              今日 API 配额已用完，采集已自动暂停。
            </p>
          ) : null}
          {isFailed ? (
            <p className="mt-3 rounded-[var(--radius-md)] border border-red-500/40 bg-red-500/10 px-3 py-2 text-[length:var(--text-sm)] text-red-600 dark:text-red-300">
              采集失败{error ? `：${error}` : ""}
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
              <CardTitle>采集流程</CardTitle>
              <CardDescription>点击节点查看对应详情；连线表示数据流向。</CardDescription>
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
                  <CardTitle>{STAGE_LABELS[selectedStage]}</CardTitle>
                  <CardDescription>
                    {panelView === "process" ? "爬取过程与 API 动态" : "当前阶段结果汇总"}
                  </CardDescription>
                </div>
                <div className="flex shrink-0 gap-1 rounded-[var(--radius-md)] border border-border bg-card/40 p-0.5">
                  <Button
                    type="button"
                    size="sm"
                    variant={panelView === "process" ? "default" : "ghost"}
                    onClick={() => setPanelView("process")}
                  >
                    过程
                  </Button>
                  <Button
                    type="button"
                    size="sm"
                    variant={panelView === "results" ? "default" : "ghost"}
                    onClick={() => setPanelView("results")}
                  >
                    结果
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

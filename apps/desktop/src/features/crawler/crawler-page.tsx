/**
 * YouTube 采集页 — 基于公共 WorkflowWorkbench（画布 + 右侧配置 + 分步日志）。
 *
 * 默认四步图（source → generate → search → summary）；左侧 palette 可继续添加节点并连线。
 * 开始采集时调用 Workflow Runtime IPC。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */

import { createContext, useCallback, useContext, useMemo, useRef, useState } from "react";
import {
  Button,
  Input,
  MarkerType,
  PageScaffold,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  WorkflowStepNode,
  WorkflowWorkbench,
  addEdge,
  applyEdgeChanges,
  applyNodeChanges,
  useTheme,
  type Connection,
  type Edge,
  type Node,
  type NodeProps,
  type NodeTypes,
  type OnEdgesChange,
  type OnNodesChange,
  type WorkflowPaletteItem,
  type WorkflowStepNodeData,
  type WorkflowStepTone,
  type XYPosition,
} from "@desk/ui";
import { useSettingsDialog } from "@feature/setting";
import type { KeywordBatchRow } from "@desk/platform/ipc/crawler";
import { useI18n, useT } from "../../i18n";
import {
  useCrawlerJob,
  type ChannelResultRow,
  type CrawlerLogRow,
  type GeneratePhase,
  type KeywordStatRow,
} from "./use-crawler-job";
import { buildCrawlerRuntimeDefinition } from "./build-crawler-runtime-definition";
import { useWorkflowRuntimeSession } from "./use-workflow-runtime-session";

/** 采集流程图步骤类型。 */
type FlowStage = "source" | "generate" | "search" | "summary";

/** i18n 翻译函数签名。 */
type Translate = (key: string, params?: Record<string, string | number>) => string;

/**
 * 采集节点 data：在公共步骤节点上增加 `kind`，配置与日志按 kind 分流。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
interface CrawlerNodeData extends WorkflowStepNodeData {
  /** 采集步骤类型（配置 / 日志按此分流）。 */
  kind: FlowStage;
}

/**
 * 将 i18n locale 映射为 `toLocaleTimeString` 可用的 BCP 47 标签。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param locale - 应用内 locale（如 `zh-CN`）
 * @returns `zh-CN` 或 `en-US`
 */
function localeTag(locale: string): string {
  if (locale === "zh-CN") {
    return "zh-CN";
  }
  return "en-US";
}

/**
 * 主题解析为 React Flow `colorMode`。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param resolvedTheme - 已解析主题名；缺省按 light
 * @returns `dark` 或 `light`
 */
function colorModeFromTheme(resolvedTheme: string | undefined): "dark" | "light" {
  if (resolvedTheme === "dark") {
    return "dark";
  }
  return "light";
}

/**
 * 生成阶段文案。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param phase - 生成阶段
 * @param t - 翻译函数
 * @returns 阶段对应文案
 */
function generatePhaseText(phase: GeneratePhase, t: Translate): string {
  if (phase === "running") {
    return t("crawler.generateRunning");
  }
  if (phase === "done") {
    return t("crawler.generateDone");
  }
  if (phase === "error") {
    return t("crawler.generateError");
  }
  return t("crawler.generateIdle");
}

/**
 * 生成阶段节点色调。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param phase - 生成阶段
 * @returns 节点 tone
 */
function generatePhaseTone(phase: GeneratePhase): WorkflowStepTone {
  if (phase === "running") {
    return "running";
  }
  if (phase === "error") {
    return "error";
  }
  if (phase === "done") {
    return "done";
  }
  return "idle";
}

/**
 * 搜索 / 汇总共用的运行态 tone（失败 / 配额 / 忙碌）。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param isFailed - 任务失败
 * @param isQuotaStop - 配额暂停
 * @param busy - 正在跑搜索
 * @param hasProgress - 已有扫描或结果进度
 * @returns 节点 tone
 */
function runProgressTone(
  isFailed: boolean,
  isQuotaStop: boolean,
  busy: boolean,
  hasProgress: boolean,
): WorkflowStepTone {
  if (isFailed) {
    return "error";
  }
  if (isQuotaStop) {
    return "warn";
  }
  if (busy) {
    return "running";
  }
  if (hasProgress) {
    return "done";
  }
  return "idle";
}

/**
 * 汇总节点 tone（含 completed 终态）。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
function summaryToneForStatus(
  isFailed: boolean,
  isQuotaStop: boolean,
  status: string,
  busy: boolean,
  hasChannels: boolean,
): WorkflowStepTone {
  if (isFailed) {
    return "error";
  }
  if (isQuotaStop) {
    return "warn";
  }
  if (status === "completed") {
    return "done";
  }
  if (busy) {
    return "running";
  }
  if (hasChannels) {
    return "done";
  }
  return "idle";
}

/**
 * 开始按钮文案。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
function startButtonLabel(busy: boolean, aiReady: boolean, t: Translate): string {
  if (busy) {
    return t("crawler.crawling");
  }
  if (aiReady) {
    return t("crawler.startAiLoop");
  }
  return t("crawler.startCrawl");
}

/**
 * 默认四步采集画布节点。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param t - i18n 翻译函数
 * @returns 初始节点列表
 */
function createDefaultNodes(t: Translate): Node<CrawlerNodeData>[] {
  return [
    {
      id: "source",
      type: "workflowStep",
      position: { x: 40, y: 120 },
      data: {
        kind: "source",
        title: t("crawler.stage.source"),
        subtitle: t("crawler.importKeywordsFirst"),
        value: t("crawler.importCsvToStart"),
        tone: "idle",
      },
    },
    {
      id: "generate",
      type: "workflowStep",
      position: { x: 300, y: 120 },
      selected: true,
      data: {
        kind: "generate",
        title: t("crawler.stage.generate"),
        subtitle: t("crawler.needAiFields"),
        value: t("crawler.generateIdle"),
        tone: "idle",
      },
    },
    {
      id: "search",
      type: "workflowStep",
      position: { x: 560, y: 120 },
      data: {
        kind: "search",
        title: t("crawler.stage.search"),
        subtitle: t("crawler.waitingStart"),
        value: t("crawler.browsedAccepted", { scanned: "0", accepted: "0" }),
        tone: "idle",
      },
    },
    {
      id: "summary",
      type: "workflowStep",
      position: { x: 820, y: 120 },
      data: {
        kind: "summary",
        title: t("crawler.stage.summary"),
        subtitle: t("crawler.status.idle"),
        value: t("crawler.showWhenDone"),
        tone: "idle",
      },
    },
  ];
}

/**
 * 默认采集流程连线。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @returns 初始边列表
 */
function createDefaultEdges(): Edge[] {
  return [
    {
      id: "source-generate",
      source: "source",
      target: "generate",
      type: "smoothstep",
      style: { stroke: "var(--color-primary)", strokeWidth: 2 },
      markerEnd: { type: MarkerType.ArrowClosed, color: "var(--color-primary)" },
    },
    {
      id: "generate-search",
      source: "generate",
      target: "search",
      type: "smoothstep",
      style: { stroke: "var(--color-primary)", strokeWidth: 2 },
      markerEnd: { type: MarkerType.ArrowClosed, color: "var(--color-primary)" },
    },
    {
      id: "search-summary",
      source: "search",
      target: "summary",
      type: "smoothstep",
      style: { stroke: "var(--color-primary)", strokeWidth: 2 },
      markerEnd: { type: MarkerType.ArrowClosed, color: "var(--color-primary)" },
    },
  ];
}

/**
 * 左侧可添加节点模板（四类采集步骤）。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param t - i18n
 * @returns palette 条目
 */
function createPaletteItems(t: Translate): WorkflowPaletteItem[] {
  return [
    {
      id: "source",
      label: t("crawler.stage.source"),
      description: t("crawler.palette.sourceDesc"),
      defaultData: {
        kind: "source",
        title: t("crawler.stage.source"),
        subtitle: t("crawler.importKeywordsFirst"),
        value: t("crawler.importCsvToStart"),
        tone: "idle",
      },
    },
    {
      id: "generate",
      label: t("crawler.stage.generate"),
      description: t("crawler.palette.generateDesc"),
      defaultData: {
        kind: "generate",
        title: t("crawler.stage.generate"),
        subtitle: t("crawler.needAiFields"),
        value: t("crawler.generateIdle"),
        tone: "idle",
      },
    },
    {
      id: "search",
      label: t("crawler.stage.search"),
      description: t("crawler.palette.searchDesc"),
      defaultData: {
        kind: "search",
        title: t("crawler.stage.search"),
        subtitle: t("crawler.waitingStart"),
        value: t("crawler.browsedAccepted", { scanned: "0", accepted: "0" }),
        tone: "idle",
      },
    },
    {
      id: "summary",
      label: t("crawler.stage.summary"),
      description: t("crawler.palette.summaryDesc"),
      defaultData: {
        kind: "summary",
        title: t("crawler.stage.summary"),
        subtitle: t("crawler.status.idle"),
        value: t("crawler.showWhenDone"),
        tone: "idle",
      },
    },
  ];
}

/**
 * 批次下拉展示文案。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param row - 批次行
 * @param index - 列表序号（0-based）
 * @param t - 翻译函数
 * @returns 展示标签
 */
function batchLabel(row: KeywordBatchRow, index: number, t: Translate): string {
  return t("crawler.batchLabel", {
    index: index + 1,
    count: row.keyword_count.toLocaleString(),
  });
}

/**
 * 格式化日志时间。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param iso - ISO 时间串
 * @param locale - 应用 locale
 * @returns 本地时分秒；无效则空串
 */
function formatLogTime(iso: string, locale: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return "";
  }
  return date.toLocaleTimeString(localeTag(locale), {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

/**
 * 是否为搜索阶段日志。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param row - 日志行
 * @returns 属于搜索阶段时为 true
 */
function isSearchPhaseLog(row: CrawlerLogRow): boolean {
  const text = row.message.toLowerCase();
  if (text.includes("search.list")) {
    return true;
  }
  if (text.includes("channels.list")) {
    return true;
  }
  if (text.includes("playlistitems")) {
    return true;
  }
  if (text.includes("begin keyword")) {
    return true;
  }
  if (text.includes("keyword done")) {
    return true;
  }
  if (row.phase === "search") {
    return true;
  }
  if (row.phase === "channel") {
    return true;
  }
  return false;
}

/**
 * 按当前步骤过滤日志。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param stage - 当前步骤；null 时返回全部
 * @param logs - 全量日志
 * @returns 过滤后的日志
 */
function filterLogsForStage(stage: FlowStage | null, logs: CrawlerLogRow[]): CrawlerLogRow[] {
  if (!stage) {
    return logs;
  }
  if (stage === "source") {
    return logs.filter((row) => row.phase === "import" || row.phase === "setup");
  }
  if (stage === "generate") {
    return logs.filter((row) => {
      if (row.phase === "generate") {
        return true;
      }
      if (row.message.toLowerCase().includes("keyword") && row.phase === "setup") {
        return true;
      }
      return false;
    });
  }
  if (stage === "search") {
    return logs.filter(isSearchPhaseLog);
  }
  return logs;
}

/**
 * 关键词进度文案。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param done - 已完成关键词数
 * @param total - 总数
 * @param active - 是否正在跑当前词
 * @returns `当前位置 / 总数`；无总数时为 `—`
 */
function formatKeywordProgress(done: number, total: number, active: boolean): string {
  if (total <= 0) {
    return "—";
  }
  let position = done;
  if (active) {
    position = done + 1;
  }
  position = Math.min(position, total);
  return `${position.toLocaleString()} / ${total.toLocaleString()}`;
}

/**
 * 将 count 输入限制在 1–200。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param raw - 原始输入
 * @returns 合法整数
 */
function clampCountPerLanguage(raw: string): number {
  const next = Number(raw);
  if (!Number.isFinite(next)) {
    return 1;
  }
  return Math.max(1, Math.min(200, next));
}

/**
 * 步骤日志空态文案。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
function stepLogsEmptyText(selectedStage: FlowStage | null, t: Translate): string {
  if (selectedStage) {
    return t("crawler.stepLogsEmpty");
  }
  return t("crawler.fullLogsEmpty");
}

/**
 * 采集页分步日志列表。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param props.rows - 日志行
 * @param props.emptyText - 空态文案
 */
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

/**
 * 频道结果列表（汇总步骤）。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param props.rows - 频道行
 */
function ChannelList({ rows }: { rows: ChannelResultRow[] }) {
  const t = useT();
  if (rows.length === 0) {
    return (
      <p className="text-[length:var(--text-sm)] text-muted-foreground">{t("crawler.channelsEmpty")}</p>
    );
  }
  return (
    <div className="space-y-2">
      {rows.map((row) => {
        const meta: React.ReactNode[] = [];
        if (row.subscriber_count != null) {
          meta.push(
            <span key="subs">
              {t("crawler.subscribers", { count: row.subscriber_count.toLocaleString() })}
            </span>,
          );
        }
        if (row.email) {
          meta.push(<span key="email">{row.email}</span>);
        }
        if (row.keyword) {
          meta.push(
            <span key="kw">{t("crawler.fromKeyword", { keyword: row.keyword })}</span>,
          );
        }
        return (
          <div
            key={row.channel_id}
            className="rounded-[var(--radius-md)] border border-border bg-card/30 px-3 py-2"
          >
            <div className="text-[length:var(--text-sm)] font-medium">{row.title}</div>
            {meta.length > 0 && (
              <div className="mt-1 flex flex-wrap gap-x-3 gap-y-1 text-xs text-muted-foreground">
                {meta}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}

/**
 * 关键词扫描统计列表（搜索步骤）。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param props.rows - 关键词统计行
 */
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

/**
 * Source 步骤：导入 CSV + 选择批次。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
function SourceConfigPanel({
  fileInputRef,
  batches,
  batchId,
  setBatchId,
  selectedBatch,
  importing,
  importCsvFile,
  importMessage,
  busy,
}: {
  fileInputRef: React.RefObject<HTMLInputElement | null>;
  batches: KeywordBatchRow[];
  batchId: string;
  setBatchId: (value: string) => void;
  selectedBatch: KeywordBatchRow | undefined;
  importing: boolean;
  importCsvFile: (file: File) => Promise<void>;
  importMessage: string;
  busy: boolean;
}) {
  const t = useT();

  let importButtonLabel = t("crawler.importKeywords");
  if (importing) {
    importButtonLabel = t("crawler.importing");
  }

  let batchSummary = t("crawler.noBatchSelected");
  if (selectedBatch) {
    batchSummary = t("crawler.keywordsReady", {
      count: selectedBatch.keyword_count.toLocaleString(),
    });
  }

  let batchHint = t("crawler.importHint");
  if (importMessage) {
    batchHint = importMessage;
  }

  let batchSelect: React.ReactNode;
  if (batches.length === 0) {
    batchSelect = (
      <Select disabled>
        <SelectTrigger>
          <SelectValue placeholder={t("crawler.importKeywordsFirst")} />
        </SelectTrigger>
      </Select>
    );
  } else {
    batchSelect = (
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
    );
  }

  return (
    <div className="space-y-4">
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
        className="w-full"
        disabled={importing || busy}
        onClick={() => fileInputRef.current?.click()}
      >
        {importButtonLabel}
      </Button>
      <label className="block space-y-1.5">
        <span className="text-[length:var(--text-sm)] text-muted-foreground">
          {t("crawler.keywordBatch")}
        </span>
        {batchSelect}
      </label>
      <div className="rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-3">
        <div className="text-xs text-muted-foreground">{t("crawler.currentBatch")}</div>
        <div className="mt-1 text-[length:var(--text-sm)] font-medium">{batchSummary}</div>
        <p className="mt-2 text-xs text-muted-foreground">{batchHint}</p>
      </div>
    </div>
  );
}

/**
 * Generate 步骤：AI 方向 / 语言 / 数量与已生成关键词预览。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
function GenerateConfigPanel({
  directions,
  setDirections,
  languages,
  setLanguages,
  countPerLanguage,
  setCountPerLanguage,
  autoLoop,
  setAutoLoop,
  busy,
  importing,
  generatePhase,
  generateMessage,
  lastGeneratedKeywords,
}: {
  directions: string;
  setDirections: (value: string) => void;
  languages: string;
  setLanguages: (value: string) => void;
  countPerLanguage: number;
  setCountPerLanguage: (value: number) => void;
  autoLoop: boolean;
  setAutoLoop: (value: boolean) => void;
  busy: boolean;
  importing: boolean;
  generatePhase: GeneratePhase;
  generateMessage: string;
  lastGeneratedKeywords: string[];
}) {
  const t = useT();
  const phaseText = generatePhaseText(generatePhase, t);

  let keywordsPreview: React.ReactNode;
  if (lastGeneratedKeywords.length === 0) {
    keywordsPreview = (
      <p className="text-[length:var(--text-sm)] text-muted-foreground">
        {t("crawler.generatedKeywordsEmpty")}
      </p>
    );
  } else {
    keywordsPreview = (
      <div className="flex flex-wrap gap-1.5">
        {lastGeneratedKeywords.slice(0, 60).map((keyword) => (
          <span
            key={keyword}
            className="rounded-[var(--radius-md)] border border-border bg-card/30 px-2 py-1 text-xs"
          >
            {keyword}
          </span>
        ))}
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-3">
        <div className="text-xs text-muted-foreground">{t("crawler.stage.generate")}</div>
        <div className="mt-1 text-[length:var(--text-sm)] font-medium">{phaseText}</div>
        {generateMessage && (
          <p className="mt-2 text-xs text-muted-foreground">{generateMessage}</p>
        )}
      </div>
      <label className="block space-y-1.5">
        <span className="text-[length:var(--text-sm)] text-muted-foreground">
          {t("crawler.aiDirections")}
        </span>
        <Input
          value={directions}
          disabled={busy || importing}
          placeholder={t("crawler.aiDirectionsPlaceholder")}
          onChange={(event) => setDirections(event.target.value)}
        />
        <span className="text-xs text-muted-foreground">{t("crawler.aiDirectionsHint")}</span>
      </label>
      <label className="block space-y-1.5">
        <span className="text-[length:var(--text-sm)] text-muted-foreground">
          {t("crawler.aiLanguages")}
        </span>
        <Input
          value={languages}
          disabled={busy || importing}
          placeholder={t("crawler.aiLanguagesPlaceholder")}
          onChange={(event) => setLanguages(event.target.value)}
        />
        <span className="text-xs text-muted-foreground">{t("crawler.aiLanguagesHint")}</span>
      </label>
      <label className="block space-y-1.5">
        <span className="text-[length:var(--text-sm)] text-muted-foreground">
          {t("crawler.aiCount")}
        </span>
        <Input
          type="number"
          min={1}
          max={200}
          value={countPerLanguage}
          disabled={busy || importing}
          onChange={(event) => setCountPerLanguage(clampCountPerLanguage(event.target.value))}
        />
      </label>
      <label className="flex items-start gap-2 text-[length:var(--text-sm)] text-muted-foreground">
        <input
          type="checkbox"
          className="mt-0.5 size-4 accent-[var(--color-primary)]"
          checked={autoLoop}
          disabled={busy}
          onChange={(event) => setAutoLoop(event.target.checked)}
        />
        <span>{t("crawler.aiAutoLoop")}</span>
      </label>
      <div>
        <div className="mb-2 text-[length:var(--text-sm)] font-medium">
          {t("crawler.generatedKeywords")}
        </div>
        {keywordsPreview}
      </div>
    </div>
  );
}

/**
 * Search 步骤：当前词、浏览/收录与按词进度。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
function SearchConfigPanel({
  currentKeyword,
  scannedCount,
  acceptedCount,
  keywordsDone,
  keywordsTotal,
  keywordStats,
  busy,
}: {
  currentKeyword: string;
  scannedCount: number;
  acceptedCount: number;
  keywordsDone: number;
  keywordsTotal: number;
  keywordStats: KeywordStatRow[];
  busy: boolean;
}) {
  const t = useT();
  let keywordDisplay = "—";
  if (currentKeyword) {
    keywordDisplay = currentKeyword;
  }

  return (
    <div className="space-y-4">
      <div className="grid grid-cols-1 gap-2">
        <div className="rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-2">
          <div className="text-xs text-muted-foreground">{t("crawler.currentKeyword")}</div>
          <div className="mt-1 truncate text-[length:var(--text-sm)] font-medium">
            {keywordDisplay}
          </div>
        </div>
        <div className="rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-2">
          <div className="text-xs text-muted-foreground">{t("crawler.browseAccept")}</div>
          <div className="mt-1 text-[length:var(--text-sm)] font-medium">
            {scannedCount.toLocaleString()} / {acceptedCount.toLocaleString()}
          </div>
        </div>
        <div className="rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-2">
          <div className="text-xs text-muted-foreground">{t("crawler.batchKeywordProgress")}</div>
          <div className="mt-1 text-[length:var(--text-sm)] font-medium tabular-nums">
            {formatKeywordProgress(keywordsDone, keywordsTotal, busy)}
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

/**
 * Summary 步骤：任务状态与频道结果。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
function SummaryConfigPanel({
  statusText,
  message,
  channelResults,
}: {
  statusText: string;
  message: string;
  channelResults: ChannelResultRow[];
}) {
  const t = useT();

  let summaryLine = message;
  if (!summaryLine) {
    summaryLine = t("crawler.waitingComplete");
  }
  if (channelResults.length > 0) {
    summaryLine = t("crawler.channelsAccepted", {
      count: channelResults.length.toLocaleString(),
    });
  }

  return (
    <div className="space-y-4">
      <div className="rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-2">
        <div className="text-xs text-muted-foreground">{t("crawler.jobStatus")}</div>
        <div className="mt-1 text-[length:var(--text-sm)] font-medium">{statusText}</div>
      </div>
      <div className="rounded-[var(--radius-md)] border border-border bg-card/40 px-3 py-2">
        <div className="text-xs text-muted-foreground">{t("crawler.summary")}</div>
        <div className="mt-1 text-[length:var(--text-sm)] font-medium">{summaryLine}</div>
      </div>
      <div>
        <div className="mb-2 text-[length:var(--text-sm)] font-medium">{t("crawler.results")}</div>
        <ChannelList rows={channelResults} />
      </div>
    </div>
  );
}

/**
 * 右侧节点配置面板（按选中步骤分流）。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
function NodeConfigPanel({
  stage,
  fileInputRef,
  batches,
  batchId,
  setBatchId,
  selectedBatch,
  importing,
  importCsvFile,
  importMessage,
  directions,
  setDirections,
  languages,
  setLanguages,
  countPerLanguage,
  setCountPerLanguage,
  autoLoop,
  setAutoLoop,
  busy,
  generatePhase,
  generateMessage,
  lastGeneratedKeywords,
  currentKeyword,
  scannedCount,
  acceptedCount,
  keywordsDone,
  keywordsTotal,
  keywordStats,
  channelResults,
  statusText,
  message,
}: {
  stage: FlowStage | null;
  fileInputRef: React.RefObject<HTMLInputElement | null>;
  batches: KeywordBatchRow[];
  batchId: string;
  setBatchId: (value: string) => void;
  selectedBatch: KeywordBatchRow | undefined;
  importing: boolean;
  importCsvFile: (file: File) => Promise<void>;
  importMessage: string;
  directions: string;
  setDirections: (value: string) => void;
  languages: string;
  setLanguages: (value: string) => void;
  countPerLanguage: number;
  setCountPerLanguage: (value: number) => void;
  autoLoop: boolean;
  setAutoLoop: (value: boolean) => void;
  busy: boolean;
  generatePhase: GeneratePhase;
  generateMessage: string;
  lastGeneratedKeywords: string[];
  currentKeyword: string;
  scannedCount: number;
  acceptedCount: number;
  keywordsDone: number;
  keywordsTotal: number;
  keywordStats: KeywordStatRow[];
  channelResults: ChannelResultRow[];
  statusText: string;
  message: string;
}) {
  const t = useT();

  if (!stage) {
    return (
      <p className="text-[length:var(--text-sm)] text-muted-foreground">
        {t("crawler.configEmptyHint")}
      </p>
    );
  }

  if (stage === "source") {
    return (
      <SourceConfigPanel
        fileInputRef={fileInputRef}
        batches={batches}
        batchId={batchId}
        setBatchId={setBatchId}
        selectedBatch={selectedBatch}
        importing={importing}
        importCsvFile={importCsvFile}
        importMessage={importMessage}
        busy={busy}
      />
    );
  }

  if (stage === "generate") {
    return (
      <GenerateConfigPanel
        directions={directions}
        setDirections={setDirections}
        languages={languages}
        setLanguages={setLanguages}
        countPerLanguage={countPerLanguage}
        setCountPerLanguage={setCountPerLanguage}
        autoLoop={autoLoop}
        setAutoLoop={setAutoLoop}
        busy={busy}
        importing={importing}
        generatePhase={generatePhase}
        generateMessage={generateMessage}
        lastGeneratedKeywords={lastGeneratedKeywords}
      />
    );
  }

  if (stage === "search") {
    return (
      <SearchConfigPanel
        currentKeyword={currentKeyword}
        scannedCount={scannedCount}
        acceptedCount={acceptedCount}
        keywordsDone={keywordsDone}
        keywordsTotal={keywordsTotal}
        keywordStats={keywordStats}
        busy={busy}
      />
    );
  }

  return (
    <SummaryConfigPanel
      statusText={statusText}
      message={message}
      channelResults={channelResults}
    />
  );
}

/**
 * 是否允许点击开始（已配置 API Key，且 AI 就绪或已有非空批次）。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
function canStartCrawl(
  apiKeyConfigured: boolean,
  aiReady: boolean,
  batchId: string,
  selectedBatch: KeywordBatchRow | undefined,
): boolean {
  if (!apiKeyConfigured) {
    return false;
  }
  if (aiReady) {
    return true;
  }
  if (!batchId) {
    return false;
  }
  if (!selectedBatch) {
    return false;
  }
  if (selectedBatch.keyword_count <= 0) {
    return false;
  }
  return true;
}

/**
 * 运行中强制聚焦的配置面板节点（固定 id，不依赖 nodes 引用）。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param generatePhase - AI 生成阶段
 * @param busy - 是否正在搜索
 * @param selectedNodeId - 用户选中节点
 * @returns 配置 / 日志面板应对准的节点 id
 */
function resolvePanelSelectedId(
  generatePhase: GeneratePhase,
  busy: boolean,
  selectedNodeId: string | null,
): string | null {
  if (generatePhase === "running") {
    return "generate";
  }
  if (busy) {
    return "search";
  }
  return selectedNodeId;
}

/**
 * 单步节点的展示覆盖（文案 / tone），不写入 React Flow nodes state。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
interface NodeDisplayOverlay {
  title: string;
  subtitle: string;
  value: string;
  tone: WorkflowStepTone;
}

/**
 * 按运行态计算各 kind 的展示覆盖。
 * 供自定义节点通过 Context 读取，避免 remap nodes 触发 StoreUpdater 死循环。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
function buildNodeDisplayByKind(params: {
  selectedBatch: KeywordBatchRow | undefined;
  aiReady: boolean;
  languages: string;
  countPerLanguage: number;
  generatePhase: GeneratePhase;
  busy: boolean;
  isFailed: boolean;
  isQuotaStop: boolean;
  acceptedCount: number;
  scannedCount: number;
  keywordsDone: number;
  keywordsTotal: number;
  currentKeyword: string;
  status: string;
  statusText: string;
  channelResults: ChannelResultRow[];
  message: string;
  nodeToneById: Record<string, WorkflowStepTone>;
  t: Translate;
}): Record<FlowStage, NodeDisplayOverlay> {
  const {
    selectedBatch,
    aiReady,
    languages,
    countPerLanguage,
    generatePhase,
    busy,
    isFailed,
    isQuotaStop,
    acceptedCount,
    scannedCount,
    keywordsDone,
    keywordsTotal,
    currentKeyword,
    status,
    statusText,
    channelResults,
    message,
    nodeToneById,
    t,
  } = params;

  let keywordCount = "0";
  if (selectedBatch) {
    keywordCount = selectedBatch.keyword_count.toLocaleString();
  }

  let sourceTone: WorkflowStepTone = "idle";
  if (selectedBatch || aiReady) {
    sourceTone = "done";
  }

  const generateTone = generatePhaseTone(generatePhase);
  const searchTone = runProgressTone(
    isFailed,
    isQuotaStop,
    busy,
    acceptedCount > 0 || scannedCount > 0,
  );
  const summaryTone = summaryToneForStatus(
    isFailed,
    isQuotaStop,
    status,
    busy,
    channelResults.length > 0,
  );

  let sourceSubtitle = t("crawler.importKeywordsFirst");
  let sourceValue = t("crawler.importCsvToStart");
  if (selectedBatch) {
    sourceSubtitle = t("crawler.wordsPending", { count: keywordCount });
    sourceValue = t("crawler.ready");
  }

  let generateSubtitle = t("crawler.needAiFields");
  if (aiReady) {
    generateSubtitle = `${languages} · ×${countPerLanguage}`;
  }

  let searchTitle = t("crawler.stage.search");
  if (busy) {
    searchTitle = t("crawler.searchingChannels");
  }
  let searchSubtitle = t("crawler.waitingStart");
  if (keywordsTotal > 0) {
    searchSubtitle = formatKeywordProgress(keywordsDone, keywordsTotal, busy);
  } else if (currentKeyword) {
    searchSubtitle = currentKeyword;
  }

  let summaryValue = t("crawler.showWhenDone");
  if (channelResults.length > 0) {
    summaryValue = t("crawler.channelsFound", {
      count: channelResults.length.toLocaleString(),
    });
  } else if (message) {
    summaryValue = message;
  }

  const source: NodeDisplayOverlay = {
    title: t("crawler.stage.source"),
    subtitle: sourceSubtitle,
    value: sourceValue,
    tone: nodeToneById.source ?? sourceTone,
  };
  const generate: NodeDisplayOverlay = {
    title: t("crawler.stage.generate"),
    subtitle: generateSubtitle,
    value: generatePhaseText(generatePhase, t),
    tone: nodeToneById.generate ?? generateTone,
  };
  const search: NodeDisplayOverlay = {
    title: searchTitle,
    subtitle: searchSubtitle,
    value: t("crawler.browsedAccepted", {
      scanned: scannedCount.toLocaleString(),
      accepted: acceptedCount.toLocaleString(),
    }),
    tone: nodeToneById.search ?? searchTone,
  };
  const summary: NodeDisplayOverlay = {
    title: t("crawler.stage.summary"),
    subtitle: statusText,
    value: summaryValue,
    tone: nodeToneById.summary ?? summaryTone,
  };

  return { source, generate, search, summary };
}

/**
 * 采集节点展示 Context：自定义节点只读覆盖层，不写回 RF nodes。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
const CrawlerNodeDisplayContext = createContext<{
  byKind: Record<FlowStage, NodeDisplayOverlay>;
  selectedNodeId: string | null;
} | null>(null);

/**
 * 采集步骤节点：用 Context 覆盖文案 / tone / 选中高亮。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param props - React Flow 节点 props
 * @returns 步骤节点
 */
function CrawlerWorkflowStepNode(props: NodeProps<Node<CrawlerNodeData>>) {
  const ctx = useContext(CrawlerNodeDisplayContext);
  const kind = props.data.kind;
  const overlay = ctx?.byKind[kind];
  const data = overlay
    ? {
      ...props.data,
      title: overlay.title,
      subtitle: overlay.subtitle,
      value: overlay.value,
      tone: overlay.tone,
    }
    : props.data;
  const selected = props.selected || props.id === ctx?.selectedNodeId;
  return <WorkflowStepNode {...props} data={data} selected={selected} />;
}

const CRAWLER_NODE_TYPES: NodeTypes = {
  workflowStep: CrawlerWorkflowStepNode,
};

/**
 * 顶部告警条：API Key / 配额 / 失败 / 错误。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
function CrawlerAlertBanner({
  apiKeyLoading,
  apiKeyConfigured,
  isQuotaStop,
  isFailed,
  error,
  openSettings,
}: {
  apiKeyLoading: boolean;
  apiKeyConfigured: boolean;
  isQuotaStop: boolean;
  isFailed: boolean;
  error: string | null;
  openSettings: () => void;
}) {
  const t = useT();
  const needApiKey = !apiKeyLoading && !apiKeyConfigured;
  const showBanner = needApiKey || isQuotaStop || isFailed || Boolean(error);
  if (!showBanner) {
    return null;
  }

  let failedText = t("crawler.crawlFailed");
  if (error) {
    failedText = t("crawler.crawlFailedWithError", { error });
  }

  return (
    <div className="shrink-0 space-y-2 border-b border-border/70 px-4 py-2">
      {needApiKey && (
        <p className="rounded-[var(--radius-md)] border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-[length:var(--text-sm)] text-amber-700 dark:text-amber-300">
          {t("crawler.needApiKeyPrefix")}{" "}
          <button
            type="button"
            className="cursor-pointer font-medium underline underline-offset-4"
            onClick={openSettings}
          >
            {t("crawler.settingsLink")}
          </button>{" "}
          {t("crawler.needApiKeySuffix")}
        </p>
      )}
      {isQuotaStop && (
        <p className="rounded-[var(--radius-md)] border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-[length:var(--text-sm)] text-amber-700 dark:text-amber-300">
          {t("crawler.quotaPaused")}
        </p>
      )}
      {isFailed && (
        <p className="rounded-[var(--radius-md)] border border-red-500/40 bg-red-500/10 px-3 py-2 text-[length:var(--text-sm)] text-red-600 dark:text-red-300">
          {failedText}
        </p>
      )}
      {!isFailed && error && (
        <p className="text-[length:var(--text-sm)] text-red-500">{error}</p>
      )}
    </div>
  );
}

/**
 * 可恢复 Runtime 实例横幅。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
function RecoverableRuntimeBanner({
  recoverable,
  onResume,
  onDismiss,
}: {
  recoverable: Array<{ instance_id: string; state: string }>;
  onResume: (instanceId: string) => void;
  onDismiss: (instanceId: string) => void;
}) {
  const t = useT();
  if (recoverable.length === 0) {
    return null;
  }

  return (
    <div className="shrink-0 space-y-2 border-b border-border/70 px-4 py-2">
      {recoverable.map((row) => (
        <div
          key={row.instance_id}
          className="flex flex-wrap items-center justify-between gap-2 rounded-[var(--radius-md)] border border-sky-500/40 bg-sky-500/10 px-3 py-2 text-[length:var(--text-sm)] text-sky-800 dark:text-sky-200"
        >
          <span>
            {t("crawler.runtimeResumeHint", {
              id: row.instance_id.slice(0, 8),
              state: row.state,
            })}
          </span>
          <div className="flex gap-2">
            <Button type="button" size="sm" onClick={() => void onResume(row.instance_id)}>
              {t("crawler.runtimeResume")}
            </Button>
            <Button
              type="button"
              size="sm"
              variant="ghost"
              onClick={() => onDismiss(row.instance_id)}
            >
              {t("crawler.runtimeDismiss")}
            </Button>
          </div>
        </div>
      ))}
    </div>
  );
}

/**
 * 采集工作流页面。
 *
 * 默认四步图 + 左侧 palette 增删改连；开始时 `workflow_runtime_start`。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */
export function CrawlerPage() {
  const t = useT();
  const { resolvedTheme } = useTheme();
  const { openSettings } = useSettingsDialog();
  const {
    apiKey,
    apiKeyConfigured,
    apiKeyLoading,
    batchId,
    setBatchId,
    batches,
    selectedBatch,
    importMessage,
    importing,
    importCsvFile,
    directions,
    setDirections,
    languages,
    setLanguages,
    countPerLanguage,
    setCountPerLanguage,
    autoLoop,
    setAutoLoop,
    aiReady,
    generatePhase,
    generateMessage,
    lastGeneratedKeywords,
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
    cancel,
    armForRuntimeCrawl,
  } = useCrawlerJob();

  const {
    instanceId: runtimeInstanceId,
    recoverable,
    nodeToneById,
    runtimeError,
    startRuntime,
    cancelRuntime,
    resumeRuntime,
    dismissRecoverable,
  } = useWorkflowRuntimeSession();

  const fileInputRef = useRef<HTMLInputElement>(null);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>("generate");
  const [nodes, setNodes] = useState<Node<CrawlerNodeData>[]>(() => createDefaultNodes(t));
  const [edges, setEdges] = useState<Edge[]>(() => createDefaultEdges());

  const isQuotaStop = stopReason === "quota_exceeded";
  const isFailed = status === "failed";
  const canStart = canStartCrawl(apiKeyConfigured, aiReady, batchId, selectedBatch);

  const panelSelectedId = useMemo(
    () => resolvePanelSelectedId(generatePhase, busy, selectedNodeId),
    [busy, generatePhase, selectedNodeId],
  );

  const selectedNode = nodes.find((node) => node.id === panelSelectedId);
  let selectedStage: FlowStage | null = null;
  if (selectedNode) {
    selectedStage = selectedNode.data.kind;
  }

  const nodeDisplayByKind = useMemo(
    () =>
      buildNodeDisplayByKind({
        selectedBatch,
        aiReady,
        languages,
        countPerLanguage,
        generatePhase,
        busy,
        isFailed,
        isQuotaStop,
        acceptedCount,
        scannedCount,
        keywordsDone,
        keywordsTotal,
        currentKeyword,
        status,
        statusText,
        channelResults,
        message,
        nodeToneById,
        t,
      }),
    [
      acceptedCount,
      aiReady,
      busy,
      channelResults,
      countPerLanguage,
      currentKeyword,
      generatePhase,
      isFailed,
      isQuotaStop,
      keywordsDone,
      keywordsTotal,
      languages,
      message,
      nodeToneById,
      scannedCount,
      selectedBatch,
      status,
      statusText,
      t,
    ],
  );

  const nodeDisplayContextValue = useMemo(
    () => ({ byKind: nodeDisplayByKind, selectedNodeId: panelSelectedId }),
    [nodeDisplayByKind, panelSelectedId],
  );

  /**
   * 经 Workflow Runtime IPC 启动采集图（真实 Generate / Search Executor）。
   *
   * @author Xiaoman
   * @created 2026-07-23
   */
  async function handleStart() {
    if (!apiKey.trim()) {
      return;
    }
    armForRuntimeCrawl();
    const definition = buildCrawlerRuntimeDefinition(nodes, edges, { autoLoop });
    let batchIdValue: string | null = null;
    if (batchId) {
      batchIdValue = batchId;
    }
    const context = {
      crawler: {
        directions,
        languages,
        count_per_language: countPerLanguage,
        batch_id: batchIdValue,
        api_key: apiKey,
        auto_loop: autoLoop,
        rate_limit_ms: 400,
      },
    };
    const instanceId = await startRuntime(JSON.stringify(definition), JSON.stringify(context));
    if (!instanceId) {
      await cancel();
    }
  }

  /**
   * 停止 Runtime 与当前 crawl job。
   *
   * @author Xiaoman
   * @created 2026-07-23
   */
  async function handleStop() {
    await cancelRuntime();
    await cancel();
  }

  /** React Flow 节点变更（拖拽 / 选中等）。 */
  const handleNodesChange: OnNodesChange<Node<WorkflowStepNodeData>> = useCallback(
    (changes) => {
      setNodes((current) => applyNodeChanges(changes, current) as Node<CrawlerNodeData>[]);
    },
    [],
  );

  /** React Flow 边变更。 */
  const handleEdgesChange: OnEdgesChange = useCallback((changes) => {
    setEdges((current) => applyEdgeChanges(changes, current));
  }, []);

  /** 用户连线时补齐样式与箭头。 */
  const handleConnect = useCallback((connection: Connection) => {
    setEdges((current) =>
      addEdge(
        {
          ...connection,
          type: "smoothstep",
          style: { stroke: "var(--color-primary)", strokeWidth: 2 },
          markerEnd: { type: MarkerType.ArrowClosed, color: "var(--color-primary)" },
        },
        current,
      ),
    );
  }, []);

  /**
   * 从左侧面板点击 / 拖入新增节点。
   *
   * @author Xiaoman
   * @created 2026-07-23
   *
   * @param item - palette 模板
   * @param position - 落点画布坐标
   */
  const handleAddNode = useCallback((item: WorkflowPaletteItem, position: XYPosition) => {
    const kind = (item.defaultData.kind ?? item.id) as FlowStage;
    const id = `${kind}-${crypto.randomUUID().slice(0, 8)}`;
    setNodes((current) => [
      ...current,
      {
        id,
        type: item.nodeType ?? "workflowStep",
        position,
        data: {
          ...item.defaultData,
          kind,
        } as CrawlerNodeData,
      },
    ]);
    setSelectedNodeId(id);
  }, []);

  const paletteItems = useMemo(() => createPaletteItems(t), [t]);

  const stageLogs = useMemo(
    () => filterLogsForStage(selectedStage, logs),
    [logs, selectedStage],
  );

  /** 底部日志区内容。 */
  function renderStepLogs() {
    if (selectedStage === "generate" && stageLogs.length === 0 && generateMessage) {
      return (
        <p className="text-[length:var(--text-sm)] text-muted-foreground">{generateMessage}</p>
      );
    }
    return <LogList rows={stageLogs} emptyText={stepLogsEmptyText(selectedStage, t)} />;
  }

  let stopDisabled = false;
  if (!busy && generatePhase !== "running" && !runtimeInstanceId) {
    stopDisabled = true;
  }

  return (
    <PageScaffold fill containerPadding="none" className="space-y-0">
      <RecoverableRuntimeBanner
        recoverable={recoverable}
        onResume={(instanceId) => void resumeRuntime(instanceId)}
        onDismiss={dismissRecoverable}
      />
      {runtimeError && (
        <div className="shrink-0 border-b border-border/70 px-4 py-2">
          <p className="text-[length:var(--text-sm)] text-red-500">
            {t("crawler.runtimeError", { error: runtimeError })}
          </p>
        </div>
      )}
      <CrawlerAlertBanner
        apiKeyLoading={apiKeyLoading}
        apiKeyConfigured={apiKeyConfigured}
        isQuotaStop={isQuotaStop}
        isFailed={isFailed}
        error={error}
        openSettings={openSettings}
      />

      <CrawlerNodeDisplayContext.Provider value={nodeDisplayContextValue}>
        <WorkflowWorkbench
          nodes={nodes as Node<WorkflowStepNodeData>[]}
          edges={edges}
          selectedNodeId={panelSelectedId}
          onSelectedNodeIdChange={setSelectedNodeId}
          onNodesChange={handleNodesChange}
          onEdgesChange={handleEdgesChange}
          onConnect={handleConnect}
          onAddNode={handleAddNode}
          paletteItems={paletteItems}
          paletteTitle={t("crawler.paletteTitle")}
          paletteHint={t("crawler.paletteHint")}
          nodeTypes={CRAWLER_NODE_TYPES}
          colorMode={colorModeFromTheme(resolvedTheme)}
          canvasTitle={t("crawler.flowTitle")}
          canvasDescription={t("crawler.flowDescription")}
          configTitle={t("crawler.configPanelTitle")}
          logsTitle={t("crawler.stepLogsTitle")}
          emptyConfig={
            <p className="text-[length:var(--text-sm)] text-muted-foreground">
              {t("crawler.configEmptyHint")}
            </p>
          }
          emptyLogs={
            <p className="text-[length:var(--text-sm)] text-muted-foreground">
              {t("crawler.stepLogsEmpty")}
            </p>
          }
          renderConfig={() => (
            <NodeConfigPanel
              stage={selectedStage}
              fileInputRef={fileInputRef}
              batches={batches}
              batchId={batchId}
              setBatchId={setBatchId}
              selectedBatch={selectedBatch}
              importing={importing}
              importCsvFile={importCsvFile}
              importMessage={importMessage}
              directions={directions}
              setDirections={setDirections}
              languages={languages}
              setLanguages={setLanguages}
              countPerLanguage={countPerLanguage}
              setCountPerLanguage={setCountPerLanguage}
              autoLoop={autoLoop}
              setAutoLoop={setAutoLoop}
              busy={busy}
              generatePhase={generatePhase}
              generateMessage={generateMessage}
              lastGeneratedKeywords={lastGeneratedKeywords}
              currentKeyword={currentKeyword}
              scannedCount={scannedCount}
              acceptedCount={acceptedCount}
              keywordsDone={keywordsDone}
              keywordsTotal={keywordsTotal}
              keywordStats={keywordStats}
              channelResults={channelResults}
              statusText={statusText}
              message={message}
            />
          )}
          renderLogs={renderStepLogs}
          footer={
            <>
              <Button type="button" variant="outline" disabled={stopDisabled} onClick={() => void handleStop()}>
                {t("crawler.stop")}
              </Button>
              <Button type="button" disabled={busy || !canStart} onClick={() => void handleStart()}>
                {startButtonLabel(busy, aiReady, t)}
              </Button>
            </>
          }
        />
      </CrawlerNodeDisplayContext.Provider>
    </PageScaffold>
  );
}

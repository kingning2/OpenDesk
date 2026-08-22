import { CheckCircle, ChevronRight, Loader2, XCircle } from "@desk/ui/icons";
import type { MonitorRun } from "@desk/platform/ipc/xianyu-monitor";
import { formatRunTime } from "../monitor-console";

export function RunRecordRow({ run, onOpen }: { run: MonitorRun; onOpen: () => void }) {
  const running = run.status === "running";
  return (
    <li>
      <button
        type="button"
        onClick={onOpen}
        className="flex w-full items-center gap-2 rounded-lg border border-border bg-card px-3 py-2 text-left transition-colors hover:bg-muted/40"
      >
        {running ? (
          <Loader2 className="size-4 shrink-0 animate-spin text-primary" />
        ) : run.status === "success" ? (
          <CheckCircle className="size-4 shrink-0 text-emerald-500" />
        ) : (
          <XCircle className="size-4 shrink-0 text-destructive" />
        )}
        <span className="min-w-0 flex-1 space-y-0.5">
          <span className="block text-xs font-medium">
            {formatRunTime(run.startedAt)}
            {running ? " · 运行中" : run.status === "success" ? " · 成功" : " · 失败"}
          </span>
          <span className="block text-[10px] text-muted-foreground">
            扫描 {run.scanned} · 新增 {run.newItems} · 推荐 {run.recommended}
            {run.error ? ` · ${run.error}` : ""}
          </span>
        </span>
        <ChevronRight className="size-4 shrink-0 text-muted-foreground" />
      </button>
    </li>
  );
}

export interface RunRecordsSectionProps {
  runs: MonitorRun[];
  runningTaskId: string | null;
  onOpen: (runId: string) => void;
}

/** 运行记录列表。 */
export function RunRecordsSection({ runs, runningTaskId, onOpen }: RunRecordsSectionProps) {
  return (
    <section className="space-y-3">
      <h2 className="text-sm font-medium">运行记录</h2>
      {runs.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          {runningTaskId
            ? "正在执行…"
            : "暂无运行记录。点击「立即运行」进入详情页查看实时过程，或从历史运行记录进入。"}
        </p>
      ) : (
        <ol className="space-y-2">
          {runs.map((run) => (
            <RunRecordRow key={run.id} run={run} onOpen={() => onOpen(run.id)} />
          ))}
        </ol>
      )}
    </section>
  );
}

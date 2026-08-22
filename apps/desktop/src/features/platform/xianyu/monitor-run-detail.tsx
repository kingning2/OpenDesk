/**
 * 闲鱼监控运行详情页 — agent 式转录（左：发送给 AI / AI 返回 / 爬虫内容） + 右：商品。
 * 双面板内部滚动（DOM 级），页面不滚动。
 */

import { OWNER_ID } from "@desk/platform/constants";
import { useCallback, useEffect, useRef, useState } from "react";
import { Button, Loading, PageScaffold, motion, toast } from "@desk/ui";
import { ArrowLeft, ExternalLink, Play } from "@desk/ui/icons";
import { managePath } from "@desk/platform/compile";
import {
  monitorResultList,
  monitorRunGet,
  monitorTaskList,
  monitorTaskRun,
  type MonitorResult,
  type MonitorRun,
} from "@desk/platform/ipc/xianyu-monitor";
import { listenMonitorProgress } from "@desk/platform/events";
import { useWorkspaceNav } from "../../../app/use-workspace-tabs";
import { formatRunTime, ThinkingDots, TranscriptStep } from "./monitor-console";


const RESULT_SPRING = { type: "spring", stiffness: 360, damping: 30 } as const;

export interface XianyuMonitorRunDetailPageProps {
  runId: string;
}

function ResultCard({ item }: { item: MonitorResult }) {
  return (
    <motion.li initial={{ opacity: 0, scale: 0.96 }} animate={{ opacity: 1, scale: 1 }} transition={RESULT_SPRING}>
      <article
        className={`flex gap-3 rounded-xl border p-3 ${
          item.aiRecommended ? "border-primary/40 bg-primary/5" : "border-border bg-card"
        }`}
      >
        {item.image ? (
          <img
            src={item.image}
            alt={item.title}
            loading="lazy"
            className="h-20 w-20 shrink-0 rounded-lg border border-border object-cover"
            onError={(event) => {
              event.currentTarget.style.display = "none";
            }}
          />
        ) : null}
        <div className="min-w-0 flex-1 space-y-1">
          <a
            href={item.url}
            target="_blank"
            rel="noreferrer"
            className="line-clamp-2 text-sm font-medium hover:underline"
          >
            {item.title}
          </a>
          <div className="flex flex-wrap gap-x-3 gap-y-1 text-xs text-muted-foreground">
            <span className="font-semibold text-foreground">{item.priceText || "—"}</span>
            {item.sellerName ? <span>{item.sellerName}</span> : null}
            {item.location ? <span>{item.location}</span> : null}
          </div>
          <p className="text-xs text-muted-foreground">{item.aiReason}</p>
        </div>
        <a
          href={item.url}
          target="_blank"
          rel="noreferrer"
          className="shrink-0 text-muted-foreground hover:text-foreground"
        >
          <ExternalLink className="size-4" />
        </a>
      </article>
    </motion.li>
  );
}

/**
 * 闲鱼监控运行详情页。
 *
 * @author Xiaoman
 * @created 2026-08-22
 */
export function XianyuMonitorRunDetailPage({ runId }: XianyuMonitorRunDetailPageProps) {
  const { selectTab } = useWorkspaceNav();
  const [run, setRun] = useState<MonitorRun | null>(null);
  const [results, setResults] = useState<MonitorResult[]>([]);
  const [taskName, setTaskName] = useState("");
  const [loading, setLoading] = useState(true);
  const logRef = useRef<HTMLDivElement>(null);
  const taskIdRef = useRef<string | null>(null);

  const loadResults = useCallback(async (taskId: string) => {
    const list = await monitorResultList(OWNER_ID, taskId);
    setResults(list);
  }, []);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setRun(null);
    setResults([]);
    taskIdRef.current = null;
    void (async () => {
      try {
        const found = await monitorRunGet(OWNER_ID, runId);
        if (cancelled) return;
        if (!found) {
          toast.error("运行记录不存在");
          selectTab(managePath("monitor"));
          return;
        }
        setRun(found);
        taskIdRef.current = found.taskId;
        const tasks = await monitorTaskList(OWNER_ID);
        if (cancelled) return;
        setTaskName(
          tasks.find((task) => task.id === found.taskId)?.name ??
            found.steps[0]?.taskName ??
            "监控任务",
        );
        void monitorResultList(OWNER_ID, found.taskId)
          .then((list) => {
            if (!cancelled) setResults(list);
          })
          .catch((error) => {
            if (!cancelled) {
              toast.error(error instanceof Error ? error.message : String(error));
            }
          });
      } catch (error) {
        if (!cancelled) {
          toast.error(error instanceof Error ? error.message : String(error));
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [runId, selectTab]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listenMonitorProgress((payload) => {
      if (payload.runId === runId) {
        setRun((prev) => (prev ? { ...prev, steps: [...prev.steps, payload] } : prev));
        if (payload.stage === "finished" || payload.stage === "failed") {
          void monitorRunGet(OWNER_ID, runId)
            .then((latest) => {
              if (latest) setRun(latest);
            })
            .catch(() => undefined);
          const taskId = taskIdRef.current;
          if (taskId) {
            void loadResults(taskId);
          }
        }
        return;
      }
      // 同任务重新运行 → 跳转新 run 详情并实时流式。
      const taskId = taskIdRef.current;
      if (taskId && payload.taskId === taskId && payload.stage === "started") {
        selectTab(`${managePath("monitor")}/runs/${payload.runId}`);
      }
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [runId, selectTab, loadResults]);

  useEffect(() => {
    const el = logRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [run?.steps.length]);

  const running = run?.status === "running";

  function handleRerun() {
    if (!run) return;
    void monitorTaskRun(OWNER_ID, run.taskId).catch((error) =>
      toast.error(error instanceof Error ? error.message : String(error)),
    );
  }

  return (
    <PageScaffold
      scroll={false}
      containerPadding="none"
      header={
        <div className="flex items-center justify-between gap-3">
          <div className="flex min-w-0 items-center gap-2">
            <Button variant="ghost" size="sm" onClick={() => selectTab(managePath("monitor"))}>
              <ArrowLeft className="size-4" />
            </Button>
            <div className="min-w-0">
              <h2 className="truncate text-sm font-semibold">{taskName || "运行详情"}</h2>
              <p className="text-xs text-muted-foreground">
                {run
                  ? `${run.status === "running" ? "运行中" : run.status === "success" ? "成功" : "失败"} · ${formatRunTime(run.startedAt)}`
                  : ""}
              </p>
            </div>
          </div>
          <div className="flex shrink-0 items-center gap-2">
            <Button size="sm" onClick={handleRerun} disabled={running}>
              <Play className="mr-1.5 size-4" />
              立即运行
            </Button>
          </div>
        </div>
      }
    >
      {loading ? (
        <div className="flex h-full min-h-0 flex-1 items-center justify-center">
          <Loading />
        </div>
      ) : run ? (
        <div className="flex h-full min-h-0 flex-1">
          <section className="flex min-h-0 w-[46%] flex-col border-r border-border/60">
            <div className="flex shrink-0 items-center justify-between border-b border-border/60 px-4 py-2">
              <span className="text-xs font-medium">运行过程</span>
              <span className="text-[10px] text-muted-foreground">{run.steps.length} 条</span>
            </div>
            <div ref={logRef} className="min-h-0 flex-1 overflow-y-auto px-4 py-3">
              <div className="space-y-3">
                {run.steps.map((step, index) => (
                  <TranscriptStep key={index} step={step} />
                ))}
                {running ? <ThinkingDots /> : null}
              </div>
            </div>
          </section>

          <section className="flex min-h-0 flex-1 flex-col">
            <div className="flex shrink-0 items-center justify-between border-b border-border/60 px-4 py-2">
              <span className="text-xs font-medium">商品（{results.length}）</span>
            </div>
            <div className="min-h-0 flex-1 overflow-y-auto px-4 py-3">
              {results.length === 0 ? (
                <p className="text-sm text-muted-foreground">暂无商品结果。</p>
              ) : (
                <ul className="space-y-3">
                  {results.map((item) => (
                    <ResultCard key={item.id} item={item} />
                  ))}
                </ul>
              )}
            </div>
          </section>
        </div>
      ) : null}
    </PageScaffold>
  );
}

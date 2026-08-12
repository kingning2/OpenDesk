/**
 * 日志面板 — 底部停靠，展示 Rust / Python 统一运行日志，支持级别过滤与自动滚动。
 *
 * Python 侧车日志经 log_pipe 转发为 tracing 事件后，与 Rust 日志一同进入
 * 进程内缓冲（logging.rs），本面板通过 `log_recent` 轮询读取。
 */

import { useEffect, useMemo, useRef } from "react";
import { IconButton } from "@desk/ui";
import { Terminal, Trash2, X } from "@desk/ui/icons";
import { logRecent } from "@desk/platform/ipc/log";
import { useLogStore, type LogLevelFilter } from "./use-log-store";

const LEVEL_RANK: Record<string, number> = {
  TRACE: 0,
  DEBUG: 1,
  INFO: 2,
  WARN: 3,
  ERROR: 4,
};

const FILTER_RANK: Record<LogLevelFilter, number> = {
  all: 0,
  debug: 1,
  info: 2,
  warn: 3,
  error: 4,
};

const FILTER_OPTIONS: { value: LogLevelFilter; label: string }[] = [
  { value: "all", label: "全部" },
  { value: "debug", label: "调试" },
  { value: "info", label: "信息" },
  { value: "warn", label: "警告" },
  { value: "error", label: "错误" },
];

/** 等级颜色（错误红 / 警告黄 / 信息绿 / 调试灰）。 */
function levelColor(level: string): string {
  switch (level) {
    case "ERROR":
    case "CRITICAL":
      return "text-destructive";
    case "WARN":
    case "WARNING":
      return "text-amber-500";
    case "DEBUG":
    case "TRACE":
      return "text-muted-foreground/60";
    default:
      return "text-emerald-500";
  }
}

/** 等级标签（英文）。 */
function levelLabel(level: string): string {
  switch (level) {
    case "ERROR":
    case "CRITICAL":
      return "ERROR";
    case "WARN":
    case "WARNING":
      return "WARN";
    case "DEBUG":
    case "TRACE":
      return "DEBUG";
    default:
      return "INFO";
  }
}

/** 北京时间（Asia/Shanghai）YYYY-MM-DD HH:MM:SS。 */
function formatTime(ts: number): string {
  const parts = new Intl.DateTimeFormat("en-CA", {
    timeZone: "Asia/Shanghai",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  }).formatToParts(ts);
  const get = (type: string) => parts.find((part) => part.type === type)?.value ?? "";
  return `${get("year")}-${get("month")}-${get("day")} ${get("hour")}:${get("minute")}:${get("second")}`;
}

/**
 * 底部日志面板；`visible` 为 false 时不渲染。
 */
export function LogPanel() {
  const visible = useLogStore((state) => state.visible);
  const entries = useLogStore((state) => state.entries);
  const levelFilter = useLogStore((state) => state.levelFilter);
  const close = useLogStore((state) => state.close);
  const setLevelFilter = useLogStore((state) => state.setLevelFilter);
  const append = useLogStore((state) => state.append);
  const clear = useLogStore((state) => state.clear);

  const viewportRef = useRef<HTMLDivElement>(null);
  const followRef = useRef(true);
  const pollingRef = useRef(false);

  // 打开面板时先增量拉取最近日志，随后每秒轮询追加。
  // 用 append（去重合并）而非整体替换，避免清掉前端（react）推入的二维码图片日志。
  useEffect(() => {
    if (!visible) {
      return;
    }
    void logRecent(500)
      .then(append)
      .catch(() => {});
    const timer = window.setInterval(() => {
      if (pollingRef.current) {
        return;
      }
      pollingRef.current = true;
      void logRecent(500)
        .then(append)
        .catch(() => {})
        .finally(() => {
          pollingRef.current = false;
        });
    }, 1000);
    return () => window.clearInterval(timer);
  }, [visible, append]);

  // 新日志到达且用户停留在底部时自动滚动到底。
  useEffect(() => {
    const el = viewportRef.current;
    if (el && followRef.current) {
      el.scrollTop = el.scrollHeight;
    }
  }, [entries]);

  const filtered = useMemo(() => {
    const rank = FILTER_RANK[levelFilter];
    if (rank === 0) {
      return entries;
    }
    return entries.filter((entry) => (LEVEL_RANK[entry.level] ?? 2) >= rank);
  }, [entries, levelFilter]);

  if (!visible) {
    return null;
  }

  return (
    <div className="flex h-64 shrink-0 flex-col border-t border-border/70 bg-card">
      <div className="flex shrink-0 items-center gap-3 border-b border-border/70 px-4 py-2">
        <span className="flex items-center gap-2 text-[length:var(--text-sm)] font-semibold">
          <Terminal className="size-3.5 text-muted-foreground" aria-hidden />
          运行日志
        </span>
        <div className="flex items-center gap-1">
          {FILTER_OPTIONS.map((option) => (
            <button
              key={option.value}
              type="button"
              onClick={() => setLevelFilter(option.value)}
              className={`rounded-[var(--radius-sm)] px-2 py-0.5 text-[length:var(--text-xs)] transition-colors ${
                levelFilter === option.value
                  ? "bg-muted/70 text-foreground"
                  : "text-muted-foreground hover:bg-muted/40 hover:text-foreground"
              }`}
            >
              {option.label}
            </button>
          ))}
        </div>
        <div className="ml-auto flex items-center gap-1">
          <IconButton label="清空日志" title="清空日志" onClick={() => void clear()}>
            <Trash2 className="size-3.5" />
          </IconButton>
          <IconButton label="关闭日志" title="关闭日志" onClick={close}>
            <X className="size-3.5" />
          </IconButton>
        </div>
      </div>

      <div
        ref={viewportRef}
        onScroll={() => {
          const el = viewportRef.current;
          if (!el) {
            return;
          }
          followRef.current = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
        }}
        className="min-h-0 flex-1 overflow-y-auto px-4 py-2 font-mono text-[length:var(--text-xs)] leading-relaxed"
      >
        {filtered.length === 0 ? (
          <p className="py-8 text-center text-muted-foreground">暂无日志</p>
        ) : (
          filtered.map((entry, index) =>
            entry.message.startsWith("data:image/") ? (
              <div key={`${entry.ts}-${index}`} className="py-1">
                <span className={`font-semibold ${levelColor(entry.level)}`}>
                  {levelLabel(entry.level)}
                </span>{" "}
                <span className="text-muted-foreground/60">{formatTime(entry.ts)}</span>{" "}
                <span className="opacity-80">{entry.source} 二维码</span>
                <img
                  src={entry.message}
                  alt="登录二维码"
                  className="mt-1 block max-h-48 rounded-[var(--radius-md)] border border-border/70 bg-white object-contain"
                />
              </div>
            ) : (
              <p
                key={`${entry.ts}-${index}`}
                className="whitespace-pre-wrap break-words text-muted-foreground"
              >
                <span className={`font-semibold ${levelColor(entry.level)}`}>
                  {levelLabel(entry.level)}
                </span>{" "}
                <span className="text-muted-foreground/60">{formatTime(entry.ts)}</span>{" "}
                <span className="opacity-80">{entry.source}</span> {entry.message}
              </p>
            ),
          )
        )}
      </div>
    </div>
  );
}

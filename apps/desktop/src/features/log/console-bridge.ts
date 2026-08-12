/**
 * 前端 console 桥接 — 把 React 侧的 console 日志写入日志 store，
 * 与 Rust / Python 日志在面板中统一展示（source = react）。
 */

import { useLogStore } from "./use-log-store";

let installed = false;

const LEVEL_MAP: Record<string, string> = {
  log: "INFO",
  info: "INFO",
  warn: "WARN",
  error: "ERROR",
  debug: "DEBUG",
};

function stringify(value: unknown): string {
  if (typeof value === "string") {
    return value;
  }
  if (value instanceof Error) {
    return value.message;
  }
  try {
    return JSON.stringify(value) ?? String(value);
  } catch {
    return String(value);
  }
}

/**
 * 安装 console 包装器（幂等）。建议在应用入口调用一次。
 */
export function installConsoleBridge(): void {
  if (installed || typeof console === "undefined") {
    return;
  }
  installed = true;

  for (const name of ["log", "info", "warn", "error", "debug"] as const) {
    const original = console[name].bind(console);
    console[name] = (...args: unknown[]) => {
      try {
        const message = args.map(stringify).join(" ").trim();
        if (message) {
          useLogStore.getState().append([
            {
              ts: Date.now(),
              level: LEVEL_MAP[name],
              source: "react",
              target: "console",
              message,
            },
          ]);
        }
      } catch {
        // 桥接自身出错不阻塞原 console。
      }
      original(...args);
    };
  }
}

/**
 * 从 pnpm lifecycle 脚本内可靠地再次调用 pnpm。
 *
 * Windows + conda 下 `shell: true` 的 cmd 常常找不到 PATH 里的 pnpm；
 * 优先用 `npm_execpath`（pnpm.cjs）经 node 直接拉起。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */
import { existsSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { platform } from "node:os";

/**
 * @param {string} command
 * @returns {boolean}
 */
function commandExists(command) {
  const checker = platform() === "win32" ? "where" : "which";
  const result = spawnSync(checker, [command], {
    shell: true,
    stdio: "ignore",
  });
  return result.status === 0;
}

/**
 * @returns {string | null}
 */
function resolveFromPath(command) {
  const checker = platform() === "win32" ? "where" : "which";
  const result = spawnSync(checker, [command], {
    shell: true,
    encoding: "utf8",
  });
  if (result.status !== 0) {
    return null;
  }
  return (
    (result.stdout ?? "")
      .split(/\r?\n/)
      .map((line) => line.trim())
      .find(Boolean) ?? null
  );
}

/**
 * @returns {{ command: string, prefixArgs: string[], shell: boolean }}
 */
export function resolvePnpmInvocation() {
  const execpath = process.env.npm_execpath?.trim();
  const userAgent = process.env.npm_config_user_agent ?? "";

  if (
    execpath &&
    existsSync(execpath) &&
    (/pnpm(\.cjs|\.js|\.mjs)?$/i.test(execpath) || userAgent.includes("pnpm"))
  ) {
    return {
      command: process.execPath,
      prefixArgs: [execpath],
      shell: false,
    };
  }

  const onPath = resolveFromPath("pnpm");
  if (onPath) {
    return {
      command: onPath,
      prefixArgs: [],
      shell: false,
    };
  }

  if (commandExists("pnpm")) {
    return {
      command: "pnpm",
      prefixArgs: [],
      shell: true,
    };
  }

  console.error(
    "[dingda] pnpm not found. Run this script via `pnpm tauri dev`, or install pnpm globally.",
  );
  process.exit(1);
}

/**
 * @param {string[]} pnpmArgs
 * @param {import("node:child_process").SpawnSyncOptionsWithStringEncoding} [options]
 * @returns {import("node:child_process").SpawnSyncReturns<string>}
 */
export function spawnPnpm(pnpmArgs, options = {}) {
  const { command, prefixArgs, shell } = resolvePnpmInvocation();
  return spawnSync(command, [...prefixArgs, ...pnpmArgs], {
    stdio: "inherit",
    shell,
    ...options,
  });
}

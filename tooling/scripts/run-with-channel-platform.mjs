/**
 * 在指定渠道平台环境下运行子命令（供 vite 等非 tauri 入口复用）。
 *
 * Tauri 请直接用：`pnpm tauri dev [platforms] [locked]`
 *
 * 用法：`node tooling/scripts/run-with-channel-platform.mjs <platforms> <command...>`
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

import { spawn } from "node:child_process";

import { applyChannelPlatformEnv } from "./read-channel-platform.mjs";

const [, , platformsArg, ...command] = process.argv;

if (!platformsArg || command.length === 0) {
  console.error(
    "用法: node tooling/scripts/run-with-channel-platform.mjs <platforms> <command...>",
  );
  process.exit(1);
}

const env = applyChannelPlatformEnv({ ...process.env }, platformsArg);

const child = spawn(command[0], command.slice(1), {
  stdio: "inherit",
  shell: true,
  env,
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code ?? 0);
});

/**
 * 在指定渠道平台环境下运行子命令（跨平台设置 OPENDESK_CHANNEL_PLATFORM）。
 *
 * 用法：`node tooling/scripts/run-with-channel-platform.mjs <platform> <command...>`
 * 示例：`node tooling/scripts/run-with-channel-platform.mjs xianyu vite`
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

import { spawn } from "node:child_process";

import { readChannelPlatformsConfig } from "./read-channel-platform.mjs";

const [, , platform, ...command] = process.argv;

if (!platform || command.length === 0) {
  console.error(
    "用法: node tooling/scripts/run-with-channel-platform.mjs <platform> <command...>",
  );
  process.exit(1);
}

const config = readChannelPlatformsConfig();
const valid = config.platforms.map((item) => item.id);
if (!valid.includes(platform)) {
  console.error(`未知平台 ${platform}，可选: ${valid.join(", ")}`);
  process.exit(1);
}

process.env.OPENDESK_CHANNEL_PLATFORM = platform;

const child = spawn(command[0], command.slice(1), {
  stdio: "inherit",
  shell: true,
  env: process.env,
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code ?? 0);
});

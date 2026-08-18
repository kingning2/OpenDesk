/**
 * 读取编译期渠道平台配置 — 供 Vite / Node 构建脚本复用。
 *
 * 环境变量 `DINGDA_CHANNEL_PLATFORM` 优先；缺省时使用 JSON 中的 `default`。
 *
 * 注意：本文件为 `.mjs`（纯 JavaScript），类型仅用 JSDoc 描述，不可写 TS 语法。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * @typedef {"xianyu" | "xiaohongshu" | "douyin"} ChannelPlatformId
 */

/**
 * @typedef {object} ChannelPlatformEntry
 * @property {ChannelPlatformId} id
 * @property {string} name
 * @property {boolean} implemented
 */

/**
 * @typedef {object} ChannelPlatformsConfig
 * @property {ChannelPlatformId} default
 * @property {ChannelPlatformEntry[]} platforms
 */

const CONFIG_PATH = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../config/channel-platforms.json",
);

/**
 * 读取 `tooling/config/channel-platforms.json`。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @returns {ChannelPlatformsConfig} 解析后的配置
 */
export function readChannelPlatformsConfig() {
  const raw = fs.readFileSync(CONFIG_PATH, "utf8");
  return /** @type {ChannelPlatformsConfig} */ (JSON.parse(raw));
}

/**
 * 解析当前构建应使用的渠道平台 id。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param {NodeJS.ProcessEnv} [env] 环境变量对象，默认 `process.env`
 * @returns {ChannelPlatformId} 校验后的平台 id
 */
export function resolveChannelPlatform(env = process.env) {
  const config = readChannelPlatformsConfig();
  const requested = env.DINGDA_CHANNEL_PLATFORM ?? config.default;
  const valid = config.platforms.map((item) => item.id);

  if (!valid.includes(requested)) {
    throw new Error(
      `未知 DINGDA_CHANNEL_PLATFORM=${requested}，可选: ${valid.join(", ")}`,
    );
  }

  return /** @type {ChannelPlatformId} */ (requested);
}

/**
 * 生成 Vite `define` 用的编译期常量对象。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param {NodeJS.ProcessEnv} [env] 环境变量对象，默认 `process.env`
 * @returns {Record<string, string>} 供 `defineConfig` 合并的键值
 */
export function channelPlatformDefine(env = process.env) {
  const platform = resolveChannelPlatform(env);
  return {
    __DINGDA_CHANNEL_PLATFORM__: JSON.stringify(platform),
  };
}

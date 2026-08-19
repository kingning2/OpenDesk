/**
 * 检测 sccache 并配置 `RUSTC_WRAPPER`，加速 Rust 增量/跨分支编译。
 *
 * 缓存目录默认 `.tools/sccache`（gitignore）。仅缓存 rustc 编译，不缓存
 * OpenSSL / BoringSSL 等 C/C++ 构建。
 *
 * 环境变量：
 * - `DINGDA_SCCACHE=0` — 禁用
 * - `DINGDA_SCCACHE=1` — 强制启用（未安装则提示）
 * - `SCCACHE_DIR` — 覆盖缓存根目录
 * - `SCCACHE_CACHE_SIZE` — 上限，默认 `20G`
 * - `RUSTC_WRAPPER` — 已设置则尊重用户配置
 *
 * @author Xiaoman
 * @created 2026-08-19
 */
import { existsSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import { platform } from "node:os";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const DEFAULT_CACHE_DIR = join(root, ".tools/sccache");
const DEFAULT_CACHE_SIZE = "20G";

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
  const line = (result.stdout ?? "")
    .split(/\r?\n/)
    .map((entry) => entry.trim())
    .find(Boolean);
  return line ?? null;
}

/**
 * @returns {string | null}
 */
function resolveSccacheExecutable() {
  const fromEnv = process.env.SCCACHE_PATH?.trim();
  if (fromEnv && existsSync(fromEnv)) {
    return fromEnv;
  }

  if (commandExists("sccache")) {
    return resolveFromPath("sccache");
  }

  const portableName = platform() === "win32" ? "sccache.exe" : "sccache";
  const portable = join(root, ".tools/sccache/bin", portableName);
  if (existsSync(portable)) {
    return portable;
  }

  return null;
}

/**
 * 为 Cargo / Tauri 子进程启用 sccache。
 *
 * @param {NodeJS.ProcessEnv} [env]
 * @returns {NodeJS.ProcessEnv}
 */
export function ensureSccache(env = { ...process.env }) {
  if (env.DINGDA_SCCACHE === "0") {
    return env;
  }

  if (env.RUSTC_WRAPPER?.trim()) {
    return env;
  }

  const sccacheExe = resolveSccacheExecutable();
  if (!sccacheExe) {
    if (env.DINGDA_SCCACHE === "1") {
      console.warn(
        "[dingda] DINGDA_SCCACHE=1 but sccache not found. Install: pnpm sccache:install",
      );
    }
    return env;
  }

  const cacheDir = env.SCCACHE_DIR?.trim() || DEFAULT_CACHE_DIR;
  mkdirSync(cacheDir, { recursive: true });

  env.RUSTC_WRAPPER = sccacheExe;
  env.SCCACHE_DIR = cacheDir;
  env.SCCACHE_CACHE_SIZE = env.SCCACHE_CACHE_SIZE?.trim() || DEFAULT_CACHE_SIZE;

  console.log(
    `[dingda] sccache enabled (${sccacheExe}, cache: ${cacheDir}, max: ${env.SCCACHE_CACHE_SIZE})`,
  );
  return env;
}

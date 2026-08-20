/**
 * Windows：确保 CMake 可用于 boring-sys2（wreq / BoringSSL）编译。
 *
 * 优先使用系统 PATH 中的 cmake 或 CMAKE 环境变量；否则使用/下载仓库内 `.tools/cmake` 便携版。
 * Cargo 只能拉 Rust crate，装不了系统工具，本脚本就是「构建时自动下载缺失工具」的那一步。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
import { existsSync, mkdirSync, readdirSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { spawnSync } from "node:child_process";
import { platform } from "node:os";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const CMAKE_VERSION = "4.4.2";
const CMAKE_ZIP_URL = `https://github.com/Kitware/CMake/releases/download/v${CMAKE_VERSION}/cmake-${CMAKE_VERSION}-windows-x86_64.zip`;
const PORTABLE_ROOT = join(root, ".tools/cmake");

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
 * @param {string} dir
 * @param {string} fileName
 * @returns {string | null}
 */
function findFile(dir, fileName) {
  if (!existsSync(dir)) {
    return null;
  }
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) {
      const nested = findFile(full, fileName);
      if (nested) {
        return nested;
      }
      continue;
    }
    if (entry.name.toLowerCase() === fileName.toLowerCase()) {
      return full;
    }
  }
  return null;
}

/**
 * 下载并解压便携 CMake 到 `.tools/cmake`。
 */
function downloadPortableCmake() {
  mkdirSync(PORTABLE_ROOT, { recursive: true });
  const zipPath = join(PORTABLE_ROOT, `cmake-${CMAKE_VERSION}-windows-x86_64.zip`);
  console.log(`[dingda] downloading portable CMake ${CMAKE_VERSION} for BoringSSL (wreq)...`);

  const ps = [
    "$ProgressPreference = 'SilentlyContinue'",
    `$zip = '${zipPath.replace(/'/g, "''")}'`,
    `$dest = '${PORTABLE_ROOT.replace(/'/g, "''")}'`,
    `$uri = '${CMAKE_ZIP_URL}'`,
    "Invoke-WebRequest -Uri $uri -OutFile $zip",
    "Expand-Archive -Path $zip -DestinationPath $dest -Force",
  ].join("; ");

  const result = spawnSync("powershell", ["-NoProfile", "-Command", ps], {
    stdio: "inherit",
    shell: true,
  });
  if (result.status !== 0) {
    console.error(
      "[dingda] failed to download CMake. Install manually from https://cmake.org/download/ and ensure cmake.exe is on PATH.",
    );
    process.exit(result.status ?? 1);
  }
}

/**
 * @returns {string | null}
 */
function resolvePortableCmake() {
  return findFile(PORTABLE_ROOT, "cmake.exe");
}

/**
 * 读取 PATH 环境变量（Windows 常用 `Path`）。
 *
 * @param {NodeJS.ProcessEnv} env
 * @returns {string}
 */
function readPathEnv(env) {
  return env.Path ?? env.PATH ?? process.env.Path ?? process.env.PATH ?? "";
}

/**
 * @param {NodeJS.ProcessEnv} env
 * @param {string} value
 */
function writePathEnv(env, value) {
  if (platform() === "win32") {
    env.Path = value;
  }
  env.PATH = value;
}

/**
 * @param {NodeJS.ProcessEnv} env
 * @param {string} cmakeDir
 */
function prependPathEnv(env, cmakeDir) {
  const current = readPathEnv(env);
  if (current.toLowerCase().includes(cmakeDir.toLowerCase())) {
    return;
  }
  writePathEnv(env, `${cmakeDir};${current}`);
}

/**
 * 清除因缺少 CMake 而失败的 boring-sys2 CMake 缓存。
 */
function clearStaleBoringSysCache() {
  const targetDir = join(root, "target");
  if (!existsSync(targetDir)) {
    return;
  }

  const profiles = ["debug", "release"];
  const buildRoots = profiles.map((profile) => join(targetDir, profile, "build"));
  for (const entry of readdirSync(targetDir, { withFileTypes: true })) {
    if (!entry.isDirectory()) {
      continue;
    }
    for (const profile of profiles) {
      buildRoots.push(join(targetDir, entry.name, profile, "build"));
    }
  }

  for (const buildRoot of buildRoots) {
    if (!existsSync(buildRoot)) {
      continue;
    }
    for (const entry of readdirSync(buildRoot, { withFileTypes: true })) {
      if (!entry.isDirectory() || !entry.name.startsWith("boring-sys2-")) {
        continue;
      }
      rmSync(join(buildRoot, entry.name), { recursive: true, force: true });
      console.log(`[dingda] cleared stale ${entry.name} (retry BoringSSL with CMake)`);
    }
  }
}

/**
 * 为 boring-sys2 配置 CMake 环境变量。
 *
 * @param {NodeJS.ProcessEnv} env
 * @returns {NodeJS.ProcessEnv}
 */
export function ensureCmake(env = { ...process.env }) {
  if (platform() !== "win32") {
    return env;
  }

  const configured = env.CMAKE?.trim();
  const hadConfiguredCmake = Boolean(configured && existsSync(configured));
  if (hadConfiguredCmake) {
    return env;
  }

  let cmakeExe = commandExists("cmake") ? resolveFromPath("cmake") : resolvePortableCmake();
  const downloaded = !cmakeExe;
  if (!cmakeExe) {
    downloadPortableCmake();
    cmakeExe = resolvePortableCmake();
  }

  if (!cmakeExe || !existsSync(cmakeExe)) {
    console.error(
      "[dingda] CMake is required to build wreq (BoringSSL) on Windows.",
    );
    console.error(
      "[dingda] Install from https://cmake.org/download/ or rerun `pnpm tauri dev` to auto-download into .tools/cmake",
    );
    process.exit(1);
  }

  env.CMAKE = cmakeExe;
  prependPathEnv(env, dirname(cmakeExe));
  console.log(`[dingda] using CMake at ${cmakeExe}`);

  // 仅在首次下载 CMake 时清 CMake 缓存；日常 dev/build 必须保留增量编译缓存。
  if (downloaded) {
    clearStaleBoringSysCache();
    console.log("[dingda] portable CMake installed under .tools/cmake (gitignored)");
  }

  return env;
}

const isDirectRun =
  Boolean(process.argv[1]) &&
  import.meta.url === pathToFileURL(process.argv[1]).href;

if (isDirectRun) {
  ensureCmake({ ...process.env });
}

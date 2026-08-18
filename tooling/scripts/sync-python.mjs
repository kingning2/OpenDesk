/**
 * 预同步 Python workspace 依赖，避免 tauri:dev 拉起 sidecar 时冷启动超时。
 */
import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { platform } from "node:os";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");

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
 * 在仓库根目录执行 `uv sync`。
 *
 * @param {{ cwd?: string, env?: NodeJS.ProcessEnv }} [options]
 * @returns {boolean} 是否执行了 sync（uv 不可用时返回 false）
 */
export function syncPythonWorkspace(options = {}) {
  if (!commandExists("uv")) {
    console.warn(
      "[dingda] uv not found; skip Python workspace pre-sync (sidecar will fall back to python)",
    );
    return false;
  }

  console.log("[dingda] syncing Python workspace dependencies...");
  const result = spawnSync("uv", ["sync", "--all-packages"], {
    cwd: options.cwd ?? root,
    env: options.env ?? process.env,
    stdio: "inherit",
    shell: true,
  });

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }

  return true;
}

const isDirectRun =
  Boolean(process.argv[1]) &&
  import.meta.url === pathToFileURL(process.argv[1]).href;

if (isDirectRun) {
  syncPythonWorkspace();
}

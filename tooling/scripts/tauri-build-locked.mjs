/**
 * 有锁发行构建：先编 license-verifier，再 `tauri build --features license-lock`。
 *
 * @author Xiaoman
 * @created 2026-07-16
 *
 * 用法：
 *   node tooling/scripts/tauri-build-locked.mjs
 *   node tooling/scripts/tauri-build-locked.mjs --target x86_64-pc-windows-msvc
 */
import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { platform } from "node:os";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const WINDOWS_DEFAULT_MSVC = "x86_64-pc-windows-msvc";

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd ?? root,
    stdio: "inherit",
    shell: true,
    env: options.env ?? process.env,
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

/**
 * 解析 CLI `--target`。
 *
 * @param {string[]} argv
 * @returns {string | null}
 */
function parseCliTarget(argv) {
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--target" && argv[index + 1]) {
      return argv[index + 1];
    }
    if (arg.startsWith("--target=")) {
      return arg.slice("--target=".length);
    }
  }
  return null;
}

const cliTarget = parseCliTarget(process.argv.slice(2));

const env = { ...process.env };
if (platform() === "win32") {
  env.RUSTUP_TOOLCHAIN =
    env.RUSTUP_TOOLCHAIN ?? "stable-x86_64-pc-windows-msvc";
  if (cliTarget) {
    env.CARGO_BUILD_TARGET = cliTarget.includes("windows-gnu")
      ? cliTarget.replace("windows-gnu", "windows-msvc")
      : cliTarget;
  } else if (!env.CARGO_BUILD_TARGET) {
    env.CARGO_BUILD_TARGET = WINDOWS_DEFAULT_MSVC;
  }
}

const verifierArgs = ["tooling/scripts/build-license-verifier.mjs"];
if (cliTarget) {
  verifierArgs.push("--target", cliTarget);
}
run("node", verifierArgs, { env });

const tauriArgs = ["tauri", "build", "--features", "license-lock"];
if (cliTarget) {
  tauriArgs.push("--target", cliTarget);
}
run("pnpm", tauriArgs, { env });

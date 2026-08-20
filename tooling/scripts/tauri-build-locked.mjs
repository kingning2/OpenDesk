/**
 * 有锁发行构建：先编 license-verifier，再 `tauri build --features license-lock`。
 *
 * @author coisini
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
import { ensureNasm } from "./ensure-nasm.mjs";
import { ensureCmake } from "./ensure-cmake.mjs";
import { ensureSccache } from "./ensure-sccache.mjs";
import { spawnPnpm } from "./spawn-pnpm.mjs";
import { syncContracts } from "./sync-contracts.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const WINDOWS_DEFAULT_MSVC = "x86_64-pc-windows-msvc";

function runPnpm(args, options = {}) {
  const result = spawnPnpm(args, {
    cwd: options.cwd ?? root,
    env: options.env ?? process.env,
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

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

const env = ensureSccache({ ...process.env });
/** @type {string | null} */
let buildTarget = cliTarget;
if (platform() === "win32") {
  ensureNasm(env);
  ensureCmake(env);
  env.RUSTUP_TOOLCHAIN =
    env.RUSTUP_TOOLCHAIN ?? "stable-x86_64-pc-windows-msvc";
  if (cliTarget) {
    buildTarget = cliTarget.includes("windows-gnu")
      ? cliTarget.replace("windows-gnu", "windows-msvc")
      : cliTarget;
  } else {
    buildTarget = env.CARGO_BUILD_TARGET || WINDOWS_DEFAULT_MSVC;
  }
  env.CARGO_BUILD_TARGET = buildTarget;
}

const verifierArgs = ["tooling/scripts/build-license-verifier.mjs"];
if (buildTarget) {
  verifierArgs.push("--target", buildTarget);
}
run("node", verifierArgs, { env });
syncContracts({ cwd: root, env });

const tauriArgs = ["tauri", "build", "--features", "license-lock"];
if (buildTarget) {
  // Must match CARGO_BUILD_TARGET so the bundler finds target/<triple>/release/.
  tauriArgs.push("--target", buildTarget);
}
runPnpm(tauriArgs, { env });

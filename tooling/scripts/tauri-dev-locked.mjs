/**
 * 有锁本地开发：先同步 license-verifier，再 `tauri dev --features license-lock`。
 *
 * @author coisini
 * @created 2026-07-16
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
import { syncPythonWorkspace } from "./sync-python.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const WINDOWS_MSVC_TRIPLE = "x86_64-pc-windows-msvc";

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

function readHostTriple() {
  const result = spawnSync("rustc", ["-vV"], {
    cwd: root,
    encoding: "utf8",
    shell: true,
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
  const match = result.stdout.match(/^host: (.+)$/m);
  if (!match) {
    process.exit(1);
  }
  return match[1].trim();
}

const env = ensureSccache({ ...process.env });
/** @type {string | null} */
let buildTarget = null;
if (platform() === "win32") {
  ensureNasm(env);
  ensureCmake(env);
  env.RUSTUP_TOOLCHAIN =
    env.RUSTUP_TOOLCHAIN ?? "stable-x86_64-pc-windows-msvc";
  buildTarget = env.CARGO_BUILD_TARGET || WINDOWS_MSVC_TRIPLE;
  env.CARGO_BUILD_TARGET = buildTarget;
}

console.log("[tauri:dev:locked] syncing license-verifier…");
const verifierArgs = ["tooling/scripts/build-license-verifier.mjs", "--force"];
if (buildTarget) {
  verifierArgs.push("--target", buildTarget);
}
run("node", verifierArgs, { env });
syncContracts({ cwd: root, env });
syncPythonWorkspace({ cwd: root, env });

const triple =
  platform() === "win32"
    ? (buildTarget ?? WINDOWS_MSVC_TRIPLE)
    : (process.env.CARGO_BUILD_TARGET ?? readHostTriple());
const ext = triple.includes("windows") ? ".exe" : "";
env.LICENSE_VERIFIER_EXE = join(
  root,
  "apps/desktop/src-tauri/binaries",
  `license-verifier-${triple}${ext}`,
);
console.log(`[tauri:dev:locked] license-verifier -> ${env.LICENSE_VERIFIER_EXE}`);

const tauriArgs = ["tauri", "dev", "--features", "license-lock"];
if (buildTarget) {
  tauriArgs.push("--target", buildTarget);
}
runPnpm(tauriArgs, { env });

import { spawnSync } from "node:child_process";
import { platform } from "node:os";
import { ensureNasm } from "./ensure-nasm.mjs";
import { ensureSccache } from "./ensure-sccache.mjs";
import { spawnPnpm } from "./spawn-pnpm.mjs";
import { syncContracts } from "./sync-contracts.mjs";
import { syncPythonWorkspace } from "./sync-python.mjs";

const args = process.argv.slice(2);
// pnpm and some wrappers may inject a standalone "--" separator.
// Tauri CLI doesn't need it; forwarding it can break argument parsing.
const normalizedArgs = args.filter((arg) => arg !== "--");
const env = ensureSccache({ ...process.env });

function commandExists(command) {
  const checker = platform() === "win32" ? "where" : "which";
  const result = spawnSync(checker, [command], {
    shell: true,
    stdio: "ignore",
  });
  return result.status === 0;
}

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

/**
 * Replace an existing `--target` / `--target=` value in argv.
 *
 * @param {string[]} argv
 * @param {string} target
 * @returns {string[]}
 */
function rewriteCliTarget(argv, target) {
  const next = [...argv];
  for (let index = 0; index < next.length; index += 1) {
    const arg = next[index];
    if (arg === "--target" && next[index + 1]) {
      next[index + 1] = target;
      return next;
    }
    if (arg.startsWith("--target=")) {
      next[index] = `--target=${target}`;
      return next;
    }
  }
  next.push("--target", target);
  return next;
}

// Sidecar spawn defaults to uv; fall back to python when uv is not installed.
if (!commandExists("uv")) {
  env.DINGDA_USE_UV = "0";
}

const cliTarget = parseCliTarget(normalizedArgs);
let tauriArgs = [...normalizedArgs];

// `pnpm tauri dev|build`：先 sync 契约 codegen，再 sync Python，避免 Rust/TS 缺类型或 sidecar 冷启动超时。
if (tauriArgs[0] === "dev" || tauriArgs[0] === "build") {
  syncContracts({ env });
}
if (tauriArgs[0] === "dev") {
  syncPythonWorkspace({ env });
}

// Windows：宿主编译器固定 MSVC；交叉目标以 --target / 已有 CARGO_BUILD_TARGET 为准。
if (platform() === "win32") {
  ensureNasm(env);
  env.RUSTUP_TOOLCHAIN =
    env.RUSTUP_TOOLCHAIN || "stable-x86_64-pc-windows-msvc";
  if (cliTarget) {
    env.CARGO_BUILD_TARGET = cliTarget.includes("windows-gnu")
      ? cliTarget.replace("windows-gnu", "windows-msvc")
      : cliTarget;
    // Keep Tauri --target in sync with Cargo (bundler path uses CLI target).
    if (cliTarget !== env.CARGO_BUILD_TARGET) {
      tauriArgs = rewriteCliTarget(tauriArgs, env.CARGO_BUILD_TARGET);
    }
  } else if (!env.CARGO_BUILD_TARGET) {
    env.CARGO_BUILD_TARGET = "x86_64-pc-windows-msvc";
  }

  // CARGO_BUILD_TARGET alone puts binaries under target/<triple>/;
  // Tauri only looks there when --target is set.
  if (!parseCliTarget(tauriArgs) && env.CARGO_BUILD_TARGET) {
    tauriArgs.push("--target", env.CARGO_BUILD_TARGET);
  }
}

const result = spawnPnpm(
  ["--filter", "@desk/desktop", "exec", "tauri", ...tauriArgs],
  { env },
);

process.exit(result.status ?? 1);

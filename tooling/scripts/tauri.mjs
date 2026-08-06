import { spawnSync } from "node:child_process";
import { platform } from "node:os";

const args = process.argv.slice(2);
// pnpm and some wrappers may inject a standalone "--" separator.
// Tauri CLI doesn't need it; forwarding it can break argument parsing.
const normalizedArgs = args.filter((arg) => arg !== "--");
const env = { ...process.env };

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

const cliTarget = parseCliTarget(normalizedArgs);
let tauriArgs = [...normalizedArgs];

// Windows：宿主编译器固定 MSVC；交叉目标以 --target / 已有 CARGO_BUILD_TARGET 为准。
if (platform() === "win32") {
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

// dev 前先构建 worker sidecar（debug），保证主进程自动拉起时能拿到真二进制。
const subcommand = normalizedArgs.find((arg) => !arg.startsWith("-"));
if (subcommand === "dev") {
  const workerBuild = spawnSync(
    "node",
    ["tooling/scripts/build-worker.mjs"],
    { stdio: "inherit", shell: true, env },
  );
  if (workerBuild.status !== 0) {
    process.exit(workerBuild.status ?? 1);
  }
}

const result = spawnSync(
  "pnpm",
  ["--filter", "@desk/desktop", "exec", "tauri", ...tauriArgs],
  {
    stdio: "inherit",
    shell: true,
    env,
  },
);

process.exit(result.status ?? 1);

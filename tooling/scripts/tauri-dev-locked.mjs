import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { platform } from "node:os";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const WINDOWS_MSVC_TRIPLE = "x86_64-pc-windows-msvc";

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

const env = { ...process.env };
/** @type {string | null} */
let buildTarget = null;
if (platform() === "win32") {
  env.RUSTUP_TOOLCHAIN =
    env.RUSTUP_TOOLCHAIN ?? "stable-x86_64-pc-windows-msvc";
  buildTarget = env.CARGO_BUILD_TARGET || WINDOWS_MSVC_TRIPLE;
  env.CARGO_BUILD_TARGET = buildTarget;
}

run("node", ["tooling/scripts/build-license-verifier.mjs"], { env });

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

const tauriArgs = ["tauri", "dev", "--features", "license-lock"];
if (buildTarget) {
  tauriArgs.push("--target", buildTarget);
}
run("pnpm", tauriArgs, { env });

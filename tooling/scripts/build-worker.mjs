import { spawnSync } from "node:child_process";
import {
  chmodSync,
  cpSync,
  existsSync,
  mkdirSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { platform } from "node:os";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const binariesDir = join(root, "apps/desktop/src-tauri/binaries");

/** 本机 Windows 默认 MSVC triple（仅当未指定 --target / 环境变量时）。 */
const WINDOWS_DEFAULT_MSVC_TRIPLE = "x86_64-pc-windows-msvc";

function parseArgs(argv) {
  const options = { target: null, release: false };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--target") {
      options.target = argv[index + 1] ?? null;
      index += 1;
      continue;
    }
    if (arg.startsWith("--target=")) {
      options.target = arg.slice("--target=".length);
    }
    if (arg === "--release") {
      options.release = true;
    }
  }
  return options;
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
    console.error("failed to read rustc host triple");
    process.exit(result.status ?? 1);
  }
  const match = result.stdout.match(/^host: (.+)$/m);
  if (!match) {
    console.error("could not parse rustc host triple");
    process.exit(1);
  }
  return match[1].trim();
}

/** Windows GNU → 同架构 MSVC；保留 i686 / aarch64 等 arch。 */
function forceWindowsMsvc(triple) {
  if (triple.includes("windows-gnu")) {
    return triple.replace("windows-gnu", "windows-msvc");
  }
  if (triple.includes("windows") && !triple.includes("windows-msvc")) {
    return WINDOWS_DEFAULT_MSVC_TRIPLE;
  }
  return triple;
}

/** 解析构建目标：CLI --target > Tauri/Cargo 环境变量 > 本机默认。 */
function resolveBuildTriple(requested) {
  const fromEnv =
    process.env.TAURI_ENV_TARGET_TRIPLE ||
    process.env.CARGO_BUILD_TARGET ||
    null;
  const raw =
    requested ??
    fromEnv ??
    (platform() === "win32" ? WINDOWS_DEFAULT_MSVC_TRIPLE : readHostTriple());

  if (platform() === "win32" || raw.includes("windows")) {
    const forced = forceWindowsMsvc(raw);
    if (forced !== raw) {
      console.warn(`WARNING: rewriting Windows target ${raw} -> ${forced}`);
    }
    return forced;
  }
  return raw;
}

function artifactName(targetTriple) {
  const ext = targetTriple.includes("windows") ? ".exe" : "";
  return `opendesk-worker-${targetTriple}${ext}`;
}

function sourceBinaryName(targetTriple) {
  return targetTriple.includes("windows") ? "opendesk-worker.exe" : "opendesk-worker";
}

const { target: targetArg, release } = parseArgs(process.argv.slice(2));

const env = { ...process.env };
if (platform() === "win32") {
  env.RUSTUP_TOOLCHAIN =
    env.RUSTUP_TOOLCHAIN ?? "stable-x86_64-pc-windows-msvc";
}

const buildTriple = resolveBuildTriple(targetArg);
const profile = release ? "release" : "debug";

console.log(`building opendesk-worker for ${buildTriple} (${profile})`);
run("cargo", [
  "build",
  ...(release ? ["--release"] : []),
  "--bin",
  "opendesk-worker",
  "--target",
  buildTriple,
], { cwd: root, env });

const sourcePath = join(
  root,
  "target",
  buildTriple,
  profile,
  sourceBinaryName(buildTriple),
);
if (!existsSync(sourcePath)) {
  console.error(`opendesk-worker binary not found at ${sourcePath}`);
  process.exit(1);
}

mkdirSync(binariesDir, { recursive: true });
const destPath = join(binariesDir, artifactName(buildTriple));
cpSync(sourcePath, destPath);
if (!buildTriple.includes("windows")) {
  chmodSync(destPath, 0o755);
}
console.log(`opendesk-worker -> ${destPath}`);

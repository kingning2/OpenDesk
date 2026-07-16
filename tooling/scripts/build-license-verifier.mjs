import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import {
  chmodSync,
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { platform } from "node:os";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const subscriptionDir = join(root, "subscription");
const binariesDir = join(root, "apps/desktop/src-tauri/binaries");
const adapterGeneratedDir = join(root, "crates/adapter/generated");

/** 本机 Windows 默认 MSVC triple（仅当未指定 --target / 环境变量时）。 */
const WINDOWS_DEFAULT_MSVC_TRIPLE = "x86_64-pc-windows-msvc";

function parseArgs(argv) {
  const options = { target: null };
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

/**
 * Windows GNU → 同架构 MSVC；保留 i686 / aarch64 等 arch。
 *
 * @param {string} triple
 * @returns {string}
 */
function forceWindowsMsvc(triple) {
  if (triple.includes("windows-gnu")) {
    return triple.replace("windows-gnu", "windows-msvc");
  }
  if (triple.includes("windows") && !triple.includes("windows-msvc")) {
    return WINDOWS_DEFAULT_MSVC_TRIPLE;
  }
  return triple;
}

/**
 * 解析构建目标：CLI --target > Tauri/Cargo 环境变量 > 本机默认。
 *
 * @param {string | null} requested
 * @returns {string}
 */
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
  return `license-verifier-${targetTriple}${ext}`;
}

function sourceBinaryName(targetTriple) {
  return targetTriple.includes("windows")
    ? "license-verifier.exe"
    : "license-verifier";
}

function sha256File(filePath) {
  const hash = createHash("sha256");
  hash.update(readFileSync(filePath));
  return hash.digest("hex");
}

const { target: targetArg } = parseArgs(process.argv.slice(2));

const env = { ...process.env };
if (platform() === "win32") {
  // 宿主编译器固定 MSVC；交叉目标（如 i686）靠 --target，不要改成 gnu。
  env.RUSTUP_TOOLCHAIN =
    env.RUSTUP_TOOLCHAIN ?? "stable-x86_64-pc-windows-msvc";
  const perlRoot = join(root, "tooling", "strawberry-perl");
  const perlBin = join(perlRoot, "perl", "bin");
  const mingwBin = join(perlRoot, "c", "bin");
  if (existsSync(join(perlBin, "perl.exe"))) {
    env.Path = `${perlBin};${mingwBin};${env.Path ?? env.PATH ?? ""}`;
    env.PATH = env.Path;
  } else {
    console.warn(
      "WARNING: tooling/strawberry-perl not found; ensure perl is on PATH for vendored OpenSSL",
    );
  }
}

const buildTriple = resolveBuildTriple(targetArg);

if (!existsSync(subscriptionDir)) {
  console.error(`subscription crate missing at ${subscriptionDir}`);
  process.exit(1);
}

const cargoArgs = [
  "build",
  "--release",
  "--bin",
  "license-verifier",
  "--target",
  buildTriple,
];

console.log(`building license-verifier for ${buildTriple}`);
run("cargo", cargoArgs, { cwd: subscriptionDir, env });

const releaseDir = join(subscriptionDir, "target", buildTriple, "release");
const sourcePath = join(releaseDir, sourceBinaryName(buildTriple));
if (!existsSync(sourcePath)) {
  console.error(`license-verifier binary not found at ${sourcePath}`);
  process.exit(1);
}

mkdirSync(binariesDir, { recursive: true });
mkdirSync(adapterGeneratedDir, { recursive: true });

const destPath = join(binariesDir, artifactName(buildTriple));
cpSync(sourcePath, destPath);
if (!buildTriple.includes("windows")) {
  chmodSync(destPath, 0o755);
}
console.log(`license-verifier -> ${destPath}`);

if (platform() === "win32") {
  const staleGnu = join(
    binariesDir,
    "license-verifier-x86_64-pc-windows-gnu.exe",
  );
  if (existsSync(staleGnu)) {
    unlinkSync(staleGnu);
    console.log(`removed stale alias ${staleGnu}`);
  }
}

const digest = sha256File(destPath);
const shaPath = join(adapterGeneratedDir, "license_verifier.sha256");
writeFileSync(shaPath, `${digest}\n`, "utf8");

const attestPath = join(adapterGeneratedDir, "license_attest_key.hex");
if (!existsSync(attestPath)) {
  console.error(
    `missing ${attestPath}; subscription build.rs should write it from public_key.pem`,
  );
  process.exit(1);
}

console.log(`sha256 -> ${shaPath} (${digest})`);
console.log(`attest key -> ${attestPath}`);

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

/** Windows OpenDesk 固定使用 MSVC 产物命名。 */
const WINDOWS_MSVC_TRIPLE = "x86_64-pc-windows-msvc";

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
 * 规范化 Windows 目标：一律 MSVC，拒绝 gnu。
 *
 * @param {string | null} requested
 * @returns {string}
 */
function resolveBuildTriple(requested) {
  if (platform() !== "win32") {
    return requested ?? process.env.CARGO_BUILD_TARGET ?? readHostTriple();
  }
  const raw = requested ?? process.env.CARGO_BUILD_TARGET ?? WINDOWS_MSVC_TRIPLE;
  if (raw.includes("windows-gnu")) {
    console.warn(
      `WARNING: refusing Windows GNU target ${raw}; using ${WINDOWS_MSVC_TRIPLE}`,
    );
    return WINDOWS_MSVC_TRIPLE;
  }
  if (raw.includes("windows") && !raw.includes("windows-msvc")) {
    console.warn(
      `WARNING: non-MSVC Windows target ${raw}; using ${WINDOWS_MSVC_TRIPLE}`,
    );
    return WINDOWS_MSVC_TRIPLE;
  }
  return raw.includes("windows") ? raw : WINDOWS_MSVC_TRIPLE;
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
  // vendored OpenSSL 需要 MSVC；避免 gnu toolchain 缺 gcc。
  env.RUSTUP_TOOLCHAIN =
    env.RUSTUP_TOOLCHAIN ?? "stable-x86_64-pc-windows-msvc";
  // openssl-src Configure 需要 perl（仓库内 portable Strawberry Perl）。
  const perlRoot = join(root, "tooling", "strawberry-perl");
  const perlBin = join(perlRoot, "perl", "bin");
  const mingwBin = join(perlRoot, "c", "bin");
  if (existsSync(join(perlBin, "perl.exe"))) {
    env.Path = `${perlBin};${mingwBin};${env.Path ?? env.PATH ?? ""}`;
    env.PATH = env.Path;
  } else {
    console.warn(
      "WARNING: tooling/strawberry-perl not found; vendored OpenSSL build may fail without perl on PATH",
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

// 清理历史 gnu 别名，避免运行时扫到错误文件。
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

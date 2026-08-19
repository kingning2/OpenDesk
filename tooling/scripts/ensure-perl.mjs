/**
 * Windows：确保 Perl 可用于 openssl-src（license-verifier vendored OpenSSL）。
 *
 * 优先系统 PATH；否则使用/下载 `tooling/strawberry-perl` 便携版。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */
import { existsSync, mkdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { spawnSync } from "node:child_process";
import { platform } from "node:os";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const PERL_ROOT = join(root, "tooling/strawberry-perl");
const PERL_BIN = join(PERL_ROOT, "perl/bin");
const PERL_C_BIN = join(PERL_ROOT, "c/bin");
const PERL_EXE = join(PERL_BIN, "perl.exe");
const STRAWBERRY_ZIP_URL =
  "https://github.com/StrawberryPerl/Perl-Dist-Strawberry/releases/download/SP_54221_64bit/strawberry-perl-5.42.2.1-64bit-portable.zip";

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
 * @param {NodeJS.ProcessEnv} env
 * @returns {string}
 */
function readPathEnv(env) {
  return env.Path ?? env.PATH ?? process.env.Path ?? process.env.PATH ?? "";
}

/**
 * @param {NodeJS.ProcessEnv} env
 * @param {string} value
 */
function writePathEnv(env, value) {
  if (platform() === "win32") {
    env.Path = value;
  }
  env.PATH = value;
}

/**
 * @param {NodeJS.ProcessEnv} env
 * @param {string[]} dirs
 */
function prependPathDirs(env, dirs) {
  let current = readPathEnv(env);
  for (const dir of dirs) {
    if (!current.toLowerCase().includes(dir.toLowerCase())) {
      current = `${dir};${current}`;
    }
  }
  writePathEnv(env, current);
}

/**
 * 下载并解压便携 Strawberry Perl。
 */
function downloadPortablePerl() {
  mkdirSync(PERL_ROOT, { recursive: true });
  const zipPath = join(PERL_ROOT, "strawberry-perl.zip");
  console.log("[dingda] downloading portable Strawberry Perl for OpenSSL build...");

  const ps = [
    "$ProgressPreference = 'SilentlyContinue'",
    `$zip = '${zipPath.replace(/'/g, "''")}'`,
    `$dest = '${PERL_ROOT.replace(/'/g, "''")}'`,
    `$uri = '${STRAWBERRY_ZIP_URL}'`,
    "Invoke-WebRequest -Uri $uri -OutFile $zip",
    "tar -xf $zip -C $dest",
  ].join("; ");

  const result = spawnSync("powershell", ["-NoProfile", "-Command", ps], {
    stdio: "inherit",
    shell: true,
  });
  if (result.status !== 0) {
    console.error(
      "[dingda] failed to download Strawberry Perl. See tooling/README-openssl-windows.md",
    );
    process.exit(result.status ?? 1);
  }
}

/**
 * @param {NodeJS.ProcessEnv} env
 * @returns {NodeJS.ProcessEnv}
 */
export function ensurePerl(env = { ...process.env }) {
  if (platform() !== "win32") {
    return env;
  }

  if (commandExists("perl")) {
    return env;
  }

  if (!existsSync(PERL_EXE)) {
    downloadPortablePerl();
  }

  if (!existsSync(PERL_EXE)) {
    console.error("[dingda] Perl is required to build license-verifier (OpenSSL vendored).");
    console.error("[dingda] See tooling/README-openssl-windows.md");
    process.exit(1);
  }

  prependPathDirs(env, [PERL_BIN, PERL_C_BIN]);
  console.log(`[dingda] using Perl at ${PERL_EXE}`);
  return env;
}

const isDirectRun =
  Boolean(process.argv[1]) &&
  import.meta.url === pathToFileURL(process.argv[1]).href;

if (isDirectRun) {
  ensurePerl({ ...process.env });
}

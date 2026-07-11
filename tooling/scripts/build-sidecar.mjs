import { spawnSync } from "node:child_process";
import { chmodSync, cpSync, existsSync, mkdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const sidecarDir = join(root, "python/sidecar");
const binariesDir = join(root, "apps/desktop/src-tauri/binaries");

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

function artifactName(targetTriple) {
  const ext = targetTriple.includes("windows") ? ".exe" : "";
  return `sidecar-${targetTriple}${ext}`;
}

function pyinstallerOutputPath(targetTriple) {
  const ext = targetTriple.includes("windows") ? ".exe" : "";
  return join(sidecarDir, "dist", `sidecar${ext}`);
}

function hasCommand(command) {
  const result = spawnSync(command, ["--version"], {
    cwd: root,
    stdio: "ignore",
    shell: true,
  });
  return result.status === 0;
}

const { target: targetArg } = parseArgs(process.argv.slice(2));

const targetTriple =
  targetArg ?? process.env.CARGO_BUILD_TARGET ?? readHostTriple();
const outputName = artifactName(targetTriple);
const destPath = join(binariesDir, outputName);

if (process.env.SKIP_SIDECAR_BUILD === "1") {
  console.log("SKIP_SIDECAR_BUILD=1 — skipping sidecar freeze");
  process.exit(0);
}

if (existsSync(destPath) && process.env.FORCE_SIDECAR_BUILD !== "1") {
  console.log(`Using existing sidecar binary: ${destPath}`);
  process.exit(0);
}

console.log(`Building frozen sidecar for target: ${targetTriple}`);

if (!hasCommand("uv")) {
  console.error("uv is required to build the frozen sidecar (install: pip install uv)");
  process.exit(1);
}

run("uv", ["sync"]);
run("uv", ["run", "--directory", sidecarDir, "pyinstaller", "sidecar.spec", "--clean", "--noconfirm"]);

const builtPath = pyinstallerOutputPath(targetTriple);
if (!existsSync(builtPath)) {
  console.error(`PyInstaller output not found: ${builtPath}`);
  process.exit(1);
}

mkdirSync(binariesDir, { recursive: true });
cpSync(builtPath, destPath);

if (!targetTriple.includes("windows")) {
  chmodSync(destPath, 0o755);
}

console.log(`Copied sidecar binary -> ${destPath}`);

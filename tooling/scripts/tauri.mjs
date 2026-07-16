import { spawnSync } from "node:child_process";
import { platform } from "node:os";

const args = process.argv.slice(2);
// pnpm and some wrappers may inject a standalone "--" separator.
// Tauri CLI doesn't need it; forwarding it can break argument parsing.
const normalizedArgs = args.filter((arg) => arg !== "--");
const env = { ...process.env };

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

// Sidecar spawn defaults to uv; fall back to python when uv is not installed.
if (!commandExists("uv")) {
  env.OPENDESK_USE_UV = "0";
}

const cliTarget = parseCliTarget(normalizedArgs);

// Windows：宿主编译器固定 MSVC；交叉目标以 --target / 已有 CARGO_BUILD_TARGET 为准。
if (platform() === "win32") {
  env.RUSTUP_TOOLCHAIN =
    env.RUSTUP_TOOLCHAIN || "stable-x86_64-pc-windows-msvc";
  if (cliTarget) {
    env.CARGO_BUILD_TARGET = cliTarget.includes("windows-gnu")
      ? cliTarget.replace("windows-gnu", "windows-msvc")
      : cliTarget;
  } else if (!env.CARGO_BUILD_TARGET) {
    env.CARGO_BUILD_TARGET = "x86_64-pc-windows-msvc";
  }
}

const result = spawnSync(
  "pnpm",
  ["--filter", "@desk/desktop", "exec", "tauri", ...normalizedArgs],
  {
    stdio: "inherit",
    shell: true,
    env,
  },
);

process.exit(result.status ?? 1);

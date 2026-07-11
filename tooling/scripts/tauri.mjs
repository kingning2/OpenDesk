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

// Sidecar spawn defaults to uv; fall back to python when uv is not installed.
if (!commandExists("uv")) {
  env.OPENDESK_USE_UV = "0";
}

// Tauri on Windows requires the MSVC toolchain; this machine's rustup default host is GNU.
if (platform() === "win32") {
  env.RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-msvc";
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

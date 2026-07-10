import { spawnSync } from "node:child_process";
import { platform } from "node:os";

const args = process.argv.slice(2);
const normalizedArgs = args[0] === "--" ? args.slice(1) : args;
const env = { ...process.env };

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

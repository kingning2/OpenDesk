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

// Windows：一律 MSVC（与 license-verifier / OpenSSL vendored 一致），不用 GNU。
if (platform() === "win32") {
  env.RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-msvc";
  env.CARGO_BUILD_TARGET =
    env.CARGO_BUILD_TARGET || "x86_64-pc-windows-msvc";
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

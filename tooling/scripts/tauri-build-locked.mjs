import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { platform } from "node:os";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");

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

const env = { ...process.env };
if (platform() === "win32") {
  env.RUSTUP_TOOLCHAIN =
    env.RUSTUP_TOOLCHAIN ?? "stable-x86_64-pc-windows-msvc";
  env.CARGO_BUILD_TARGET =
    env.CARGO_BUILD_TARGET || "x86_64-pc-windows-msvc";
}

run("node", ["tooling/scripts/build-license-verifier.mjs"], { env });
run("pnpm", ["tauri", "build", "--features", "license-lock"], { env });

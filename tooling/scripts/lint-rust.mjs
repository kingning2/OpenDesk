import { spawnSync } from "node:child_process";
import { platform } from "node:os";
import { ensureSccache } from "./ensure-sccache.mjs";

function run(command, args) {
  const env = ensureSccache({ ...process.env });
  if (platform() === "win32") {
    env.RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-msvc";
  }

  const result = spawnSync(command, args, {
    stdio: "inherit",
    shell: true,
    env,
  });

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

run("cargo", ["fmt", "--all", "--", "--check"]);
run("cargo", ["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"]);

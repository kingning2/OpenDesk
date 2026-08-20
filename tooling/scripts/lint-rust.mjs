import { spawnSync } from "node:child_process";
import { platform } from "node:os";
import { ensureSccache } from "./ensure-sccache.mjs";
import { ensureNasm } from "./ensure-nasm.mjs";
import { ensureCmake } from "./ensure-cmake.mjs";

function run(command, args) {
  const env = ensureSccache({ ...process.env });
  if (platform() === "win32") {
    env.RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-msvc";
  }
  // boring-sys2 (BoringSSL) 编译需要 CMake + NASM，两个便携工具缺一不可。
  ensureNasm(env);
  ensureCmake(env);

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

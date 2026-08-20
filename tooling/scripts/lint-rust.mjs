import { existsSync, readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import { platform } from "node:os";
import { ensureSccache } from "./ensure-sccache.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const pathSep = platform() === "win32" ? ";" : ":";

function resolveNasmPath() {
  // boring-sys2 (BoringSSL) 编译需要 NASM 汇编器。若系统未安装，则
  // 回退到仓库内的便携版 .tools/nasm/<ver>/（与 sccache 同策略）。
  const base = join(root, ".tools", "nasm");
  if (!existsSync(base)) return null;
  const dirs = readdirSync(base)
    .map((name) => join(base, name))
    .filter((dir) => existsSync(join(dir, "nasm.exe")) || existsSync(join(dir, "nasm")));
  if (dirs.length === 0) return null;
  dirs.sort((a, b) => b.localeCompare(a));
  return dirs[0];
}

function run(command, args) {
  const env = ensureSccache({ ...process.env });
  if (platform() === "win32") {
    env.RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-msvc";
  }
  const nasmDir = resolveNasmPath();
  if (nasmDir) {
    env.PATH = `${nasmDir}${pathSep}${env.PATH}`;
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

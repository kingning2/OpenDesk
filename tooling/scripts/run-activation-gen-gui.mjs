/**
 * 启动 subscription 激活码签发 GUI（Slint）。
 *
 * 作者：coisini
 * 创建时间：2026-07-16
 */
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const subscription = path.join(root, "subscription");
const perlBin = path.join(root, "tooling", "strawberry-perl", "perl", "bin");
const perlC = path.join(root, "tooling", "strawberry-perl", "c", "bin");

const env = { ...process.env };
env.RUSTUP_TOOLCHAIN =
  env.RUSTUP_TOOLCHAIN || "stable-x86_64-pc-windows-msvc";
if (process.platform === "win32") {
  env.Path = `${perlBin};${perlC};${env.Path || env.PATH || ""}`;
}

const result = spawnSync(
  "cargo",
  ["run", "--release", "--features", "gui", "--bin", "activation-gen-gui"],
  {
    cwd: subscription,
    env,
    stdio: "inherit",
    shell: true,
  },
);

process.exit(result.status ?? 1);

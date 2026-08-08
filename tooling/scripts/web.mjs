import { spawn, spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { platform } from "node:os";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const WINDOWS_MSVC_TRIPLE = "x86_64-pc-windows-msvc";

const args = process.argv.slice(2);
// 解析 `pnpm web dev` / `pnpm web dev --locked` / `pnpm web build --target <triple>`。
const subcommand = args.find((arg) => !arg.startsWith("-"));
const locked = args.includes("--locked") || args.includes("-l");
const cliTarget = (() => {
  for (let index = 0; index < args.length; index += 1) {
    if (args[index] === "--target" && args[index + 1]) {
      return args[index + 1];
    }
    if (args[index].startsWith("--target=")) {
      return args[index].slice("--target=".length);
    }
  }
  return null;
})();

function run(command, argv, options = {}) {
  const result = spawnSync(command, argv, {
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
let buildTarget = null;
if (platform() === "win32") {
  env.RUSTUP_TOOLCHAIN =
    env.RUSTUP_TOOLCHAIN ?? "stable-x86_64-pc-windows-msvc";
  buildTarget = env.CARGO_BUILD_TARGET || WINDOWS_MSVC_TRIPLE;
  env.CARGO_BUILD_TARGET = buildTarget;
}

const USAGE = `\
web — OpenDesk web 端（前端 vite + 后端 axum server）

用法:
  pnpm web                       启动开发服务（vite :1422 + server :8899）
  pnpm web dev [--locked]        启动开发服务（等价于无参数）
  pnpm web build [--locked]      构建前端产物 + 后端 server 二进制
  pnpm web server [--locked]     仅启动后端 server（无前端）
  pnpm web --help                显示帮助

选项:
  --locked, -l  启用 license-lock（先构建 license-verifier，server 带该 feature）
`;

/** 并发起 vite 与 axum server；任一退出则全部清理。 */
function startDevServer(serverArgs) {
  const children = [];

  const vite = spawn("pnpm", ["--filter", "@desk/web", "dev"], {
    cwd: root,
    stdio: "inherit",
    shell: true,
    env,
  });
  children.push(vite);

  const server = spawn(
    "cargo",
    ["run", "-p", "opendesk-server", ...serverArgs],
    { cwd: root, stdio: "inherit", shell: true, env },
  );
  children.push(server);

  let exiting = false;
  function shutdown(code) {
    if (exiting) {
      return;
    }
    exiting = true;
    for (const child of children) {
      if (!child.killed) {
        child.kill();
      }
    }
    process.exit(code);
  }

  for (const child of children) {
    child.on("error", () => shutdown(1));
    child.on("exit", (code) => {
      // 任一进程退出即整体结束（vite 或 server 挂掉都没意义）。
      shutdown(code ?? 0);
    });
  }
  process.on("SIGINT", () => shutdown(0));
  process.on("SIGTERM", () => shutdown(0));
}

function main() {
  if (args.includes("--help") || args.includes("-h")) {
    process.stdout.write(USAGE);
    process.exit(0);
  }

  const serverArgs = locked ? ["--features", "license-lock"] : [];
  // 无子命令时默认启动开发服务（`pnpm web` 即 `pnpm web dev`）。
  const effective = subcommand ?? "dev";

  switch (effective) {
    case "dev": {
      // dev 前先构建 worker sidecar（debug），保证 server 自动拉起时拿到真二进制。
      run("node", ["tooling/scripts/build-worker.mjs"], { env });
      if (locked) {
        run("node", ["tooling/scripts/build-license-verifier.mjs"], { env });
      }
      startDevServer(serverArgs);
      break;
    }
    case "server": {
      if (locked) {
        run("node", ["tooling/scripts/build-license-verifier.mjs"], { env });
      }
      startDevServer(serverArgs);
      break;
    }
    case "build": {
      // 前端静态产物（dist/）。
      run("pnpm", ["--filter", "@desk/web", "build"], { env });
      // locked 需要 license-verifier sidecar（server 校验授权用）。
      if (locked) {
        const verifierArgs = ["tooling/scripts/build-license-verifier.mjs"];
        if (cliTarget) {
          verifierArgs.push("--target", cliTarget);
        }
        run("node", verifierArgs, { env });
      }
      // 后端 server 二进制（--release）。
      const cargoArgs = [
        "build",
        "-p",
        "opendesk-server",
        "--release",
        ...serverArgs,
      ];
      if (cliTarget) {
        cargoArgs.push("--target", cliTarget);
      }
      run("cargo", cargoArgs, { env });
      break;
    }
    default: {
      process.stderr.write(`未知子命令: ${subcommand}\n\n${USAGE}`);
      process.exit(1);
    }
  }
}

main();

import { spawnSync } from "node:child_process";

function hasCommand(command) {
  const result = spawnSync(command, ["--version"], { shell: true });
  return result.status === 0;
}

function run(command, args) {
  const result = spawnSync(command, args, { stdio: "inherit", shell: true });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

const useUv = hasCommand("uv");
const runner = useUv ? "uv" : "python";
const checkArgs = useUv
  ? ["run", "ruff", "check", "python"]
  : ["-m", "ruff", "check", "python"];
const formatArgs = useUv
  ? ["run", "ruff", "format", "python", "--check"]
  : ["-m", "ruff", "format", "python", "--check"];

run(runner, checkArgs);
run(runner, formatArgs);

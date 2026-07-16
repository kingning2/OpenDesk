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

function stagedFiles(pattern) {
  const result = spawnSync(
    "git",
    ["diff", "--cached", "--name-only", "--diff-filter=ACMR"],
    { encoding: "utf8" },
  );
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }

  const files = result.stdout
    .trim()
    .split(/\r?\n/)
    .filter(Boolean);

  return pattern ? files.filter((file) => pattern.test(file)) : files;
}

function blockedConfigFiles(files) {
  const protectedPatterns = [
    /^eslint\.config\.(js|mjs|cjs)$/i,
    /^tsconfig\.lint\.json$/i,
    /^rust-toolchain\.toml$/i,
    /^rustfmt\.toml$/i,
    /^clippy\.toml$/i,
    /^pyproject\.toml$/i,
    /^\.editorconfig$/i,
    /^\.gitattributes$/i,
    /^\.gitignore$/i,
    /^\.husky\/pre-commit$/i,
    /^tooling\/scripts\/pre-commit\.mjs$/i,
  ];

  return files.filter((file) =>
    protectedPatterns.some((pattern) => pattern.test(file.replaceAll("\\", "/"))),
  );
}

/** 形如 role/chore/slug 的分支允许改护栏文件（专用 tooling PR）。 */
function isToolingChoreBranch() {
  const result = spawnSync("git", ["branch", "--show-current"], {
    encoding: "utf8",
    shell: true,
  });
  if (result.status !== 0) {
    return false;
  }
  const branch = (result.stdout ?? "").trim().replaceAll("\\", "/");
  return /\/chore\//.test(branch) || branch.startsWith("chore/");
}

function stagedPatch(file) {
  const result = spawnSync("git", ["diff", "--cached", "--", file], {
    encoding: "utf8",
  });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
  return result.stdout ?? "";
}

function hasSensitivePattern(text) {
  const patterns = [
    /sk-[a-zA-Z0-9]{20,}/,
    /ghp_[a-zA-Z0-9]{30,}/,
    /AKIA[0-9A-Z]{16}/,
    /api[_-]?key\s*[:=]\s*["'][^"']+["']/i,
    /secret\s*[:=]\s*["'][^"']+["']/i,
  ];
  return patterns.some((pattern) => pattern.test(text));
}

function restage(files) {
  if (files.length === 0) {
    return;
  }
  run("git", ["add", "--", ...files]);
}

const tsFiles = stagedFiles(/\.(ts|tsx)$/);
const rsFiles = stagedFiles(/\.rs$/);
const pyFiles = stagedFiles(/\.py$/);
const allStaged = stagedFiles();
const modifiedConfig = blockedConfigFiles(allStaged);

if (modifiedConfig.length > 0 && !isToolingChoreBranch()) {
  console.error(
    "[pre-commit] Blocked: editing lint/format/git guardrail files is not allowed in normal feature commits.",
  );
  console.error(
    "[pre-commit] If this is intentional, open a dedicated `*/chore/*` tooling PR.",
  );
  for (const file of modifiedConfig) {
    console.error(` - ${file}`);
  }
  process.exit(1);
}

if (modifiedConfig.length > 0 && isToolingChoreBranch()) {
  console.warn(
    "[pre-commit] Allowing guardrail file edits on chore tooling branch:",
  );
  for (const file of modifiedConfig) {
    console.warn(` - ${file}`);
  }
}

if (tsFiles.length > 0) {
  run("eslint", ["--fix", ...tsFiles]);
  restage(tsFiles);
  run("pnpm", ["lint:types"]);
}

if (rsFiles.length > 0) {
  for (const file of rsFiles) {
    run("rustfmt", ["--edition", "2021", file]);
  }
  restage(rsFiles);
}

if (pyFiles.length > 0) {
  const useUv = hasCommand("uv");
  const runner = useUv ? "uv" : "python";
  const checkArgs = useUv
    ? ["run", "ruff", "check", "--fix", ...pyFiles]
    : ["-m", "ruff", "check", "--fix", ...pyFiles];
  const formatArgs = useUv
    ? ["run", "ruff", "format", ...pyFiles]
    : ["-m", "ruff", "format", ...pyFiles];

  run(runner, checkArgs);
  run(runner, formatArgs);
  restage(pyFiles);
}

for (const file of allStaged) {
  const patch = stagedPatch(file);
  if (hasSensitivePattern(patch)) {
    console.error(
      `[pre-commit] Potential secret detected in staged diff: ${file}. Remove it before commit.`,
    );
    process.exit(1);
  }
  if (/^\+.*\bdebugger\b/m.test(patch)) {
    console.error(`[pre-commit] 'debugger' found in staged diff: ${file}.`);
    process.exit(1);
  }
}

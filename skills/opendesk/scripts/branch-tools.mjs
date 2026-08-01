#!/usr/bin/env node

/**
 * OpenDesk 分支创建与 active-branch 规则同步。
 */

import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import { createInterface } from "node:readline/promises";
import process, { stdin, stdout } from "node:process";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const configPath = resolve(root, "skills/opendesk/config/branch_roles.json");
const activeRulePath = resolve(root, ".cursor/rules/active-branch.mdc");
const slugPattern = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

function loadConfig() {
  return JSON.parse(readFileSync(configPath, "utf8"));
}

function runGit(args, { check = true, capture = false } = {}) {
  const result = spawnSync("git", args, {
    cwd: root,
    encoding: "utf8",
    stdio: capture ? ["ignore", "pipe", "pipe"] : "inherit",
  });
  if (result.error) throw result.error;
  if (check && result.status !== 0) {
    throw new Error((result.stderr || `git ${args.join(" ")} failed`).trim());
  }
  return result;
}

function currentBranch() {
  const branch = runGit(["branch", "--show-current"], { capture: true }).stdout.trim();
  if (!branch) throw new Error("detached HEAD — checkout a branch first");
  return branch;
}

function roleKeys(config) {
  return Object.keys(config.roles).filter((key) => key !== "integration");
}

function validateKind(config, value) {
  const kind = value?.trim().toLowerCase();
  if (!kind || !config.branch_kinds[kind]) {
    throw new Error(`unknown kind ${JSON.stringify(kind)}; choose: ${Object.keys(config.branch_kinds).join(", ")}`);
  }
  return kind;
}

function validateSlug(value) {
  const slug = value?.trim().toLowerCase();
  if (!slugPattern.test(slug ?? "")) {
    throw new Error(`invalid slug ${JSON.stringify(slug)}; use kebab-case`);
  }
  return slug;
}

function parseBranch(config, branch) {
  if (branch === "main") return { roleKey: "integration" };
  const legacy = config.legacy_branch_map?.[branch];
  if (legacy) return { roleKey: legacy };
  const [roleKey, kind, slug, extra] = branch.split("/");
  if (!extra && config.roles[roleKey] && config.branch_kinds[kind] && slug) {
    return { roleKey, kind, slug };
  }
  if (!slug && config.roles[roleKey] && kind) return { roleKey, slug: kind };
  return { roleKey: "integration", slug: branch };
}

function renderList(values, empty) {
  return values?.length ? values.map((value) => `- \`${value}\``).join("\n") : empty;
}

function renderActiveRule(config, branch) {
  const { roleKey, kind, slug } = parseBranch(config, branch);
  const role = config.roles[roleKey];
  const kindConfig = kind ? config.branch_kinds[kind] : undefined;
  const kindBlock = [
    kindConfig ? `**类型：** ${kindConfig.label} (\`${kind}\`) — ${kindConfig.description}` : "",
    slug ? `**任务：** \`${slug}\`` : "",
  ].filter(Boolean).join("\n\n");
  const rules = role.rule_files?.length
    ? role.rule_files.map((name) => `- [\`.cursor/rules/${name}\`](${name})`).join("\n")
    : "- [`master.md`](master.md) only";

  return `---
description: Active branch scope for ${branch} (auto-generated — run pnpm branch:sync)
alwaysApply: true
---

# 当前分支：\`${branch}\`

**角色：** ${role.label}

${kindBlock}

${role.description}

## 允许修改

${renderList(role.allowed_globs, "- _(无)_")}

## 可选（跨端契约 — 先改 Contract 再 codegen）

${renderList(role.optional_globs, "- _(无 — 非本分支职责)_")}

## 禁止修改（除非用户明确要求扩 scope）

${renderList(role.forbidden_globs, "- _(无额外禁止路径)_")}

## 细则规则

${rules}

---

> 由 \`skills/opendesk/scripts/branch-tools.mjs\` 根据分支名生成。
> 切换分支后若约束不符，运行 \`pnpm branch:sync\`。
`;
}

function syncRule(config, branch, dryRun) {
  const content = renderActiveRule(config, branch);
  if (!dryRun) {
    mkdirSync(dirname(activeRulePath), { recursive: true });
    writeFileSync(activeRulePath, content, "utf8");
  }
}

function parseArgs(argv) {
  const options = { positional: [] };
  for (let index = 0; index < argv.length; index += 1) {
    const value = argv[index];
    if (value === "--interactive" || value === "-i") options.interactive = true;
    else if (value === "--dry-run") options.dryRun = true;
    else if (value === "--no-checkout") options.noCheckout = true;
    else if (value === "--quiet" || value === "-q") options.quiet = true;
    else if (value === "--branch") options.branch = argv[++index];
    else if (value === "--base") options.base = argv[++index];
    else options.positional.push(value);
  }
  return options;
}

async function choose(input, label, values, describe) {
  stdout.write(`\n${label}\n`);
  values.forEach((value, index) => stdout.write(`  ${index + 1}) ${value} — ${describe(value)}\n`));
  while (true) {
    const answer = (await input.question("> ")).trim().toLowerCase();
    const numeric = Number(answer);
    if (Number.isInteger(numeric) && numeric >= 1 && numeric <= values.length) return values[numeric - 1];
    if (values.includes(answer)) return answer;
    stdout.write("无效选择，请重试。\n");
  }
}

function branchExists(name) {
  return runGit(["rev-parse", "--verify", name], { check: false, capture: true }).status === 0;
}

function resolveBase(base) {
  runGit(["fetch", "origin", base], { check: false });
  if (branchExists(base)) return base;
  if (branchExists(`origin/${base}`)) return `origin/${base}`;
  throw new Error(`base branch not found: ${base}`);
}

async function createBranch(config, options) {
  let [roleKey, kind, slug] = options.positional;
  let base = options.base ?? "main";
  if (options.interactive || !roleKey) {
    const input = createInterface({ input: stdin, output: stdout });
    try {
      roleKey = await choose(input, "角色", roleKeys(config), (key) => config.roles[key].label);
      kind = await choose(input, "类型", Object.keys(config.branch_kinds), (key) => config.branch_kinds[key].label);
      slug = await input.question("任务 slug（kebab-case）: ");
      base = (await input.question(`基于分支 [${base}]: `)).trim() || base;
    } finally {
      input.close();
    }
  }
  if (!roleKeys(config).includes(roleKey)) throw new Error(`unknown role ${JSON.stringify(roleKey)}`);
  kind = validateKind(config, kind);
  slug = validateSlug(slug);
  const name = `${config.roles[roleKey].branch_prefix}/${kind}/${slug}`;
  if (branchExists(name)) throw new Error(`branch already exists: ${name}`);
  if (options.dryRun) {
    stdout.write(`[dry-run] git branch ${name} ${base}\n`);
    return;
  }
  const startPoint = resolveBase(base);
  runGit(["branch", name, startPoint]);
  if (!options.noCheckout) {
    runGit(["checkout", name]);
    syncRule(config, name, false);
  }
  stdout.write(`Created branch ${name}\n`);
}

async function main() {
  const [command = "sync", ...argv] = process.argv.slice(2);
  const options = parseArgs(argv);
  const config = loadConfig();
  if (command === "create") {
    await createBranch(config, options);
    return;
  }
  if (command !== "sync") throw new Error(`unknown command: ${command}`);
  const branch = options.branch ?? currentBranch();
  syncRule(config, branch, options.dryRun);
  if (!options.quiet) stdout.write(`${options.dryRun ? "[dry-run] " : ""}synced ${branch}\n`);
}

main().catch((error) => {
  process.stderr.write(`error: ${error.message}\n`);
  process.exitCode = 1;
});

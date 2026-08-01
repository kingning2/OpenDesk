#!/usr/bin/env node

/**
 * OpenDesk 两层架构与 Contract 一致性检查。
 */

import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, extname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import console from "node:console";
import process from "node:process";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const violations = [];
const sourceExtensions = new Set([".rs", ".ts", ".tsx", ".js", ".mjs", ".json", ".toml", ".yml", ".yaml"]);

function walk(directory, predicate = () => true) {
  if (!existsSync(directory)) return [];
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    if (["node_modules", "target", ".git", "dist", "strawberry-perl"].includes(entry.name)) return [];
    const path = join(directory, entry.name);
    return entry.isDirectory() ? walk(path, predicate) : predicate(path) ? [path] : [];
  });
}

function checkFrontendBoundary() {
  const features = resolve(root, "apps/desktop/src/features");
  for (const path of walk(features, (file) => [".ts", ".tsx"].includes(extname(file)))) {
    const text = readFileSync(path, "utf8");
    if (/from\s+["']@tauri-apps\/api/.test(text)) violations.push(`${relative(root, path)}: direct Tauri API import`);
    if (/\binvoke\s*\(/.test(text)) violations.push(`${relative(root, path)}: direct invoke() call`);
    if (/https?:\/\/(?:127\.0\.0\.1|localhost):\d+/.test(text)) violations.push(`${relative(root, path)}: direct localhost HTTP`);
    if (/new\s+WebSocket\s*\(/.test(text)) violations.push(`${relative(root, path)}: WebSocket bypass`);
  }
}

function checkZeroPythonRuntime() {
  const activeRoots = ["apps", "crates", "packages", "tooling", ".github"];
  for (const directory of activeRoots) {
    for (const path of walk(resolve(root, directory))) {
      if (extname(path) === ".py") violations.push(`${relative(root, path)}: Python file remains`);
      if (!sourceExtensions.has(extname(path))) continue;
      const text = readFileSync(path, "utf8");
      if (/\b(?:Python Sidecar|SidecarLifecycle|AgentSidecarGateway|OPENDESK_SIDECAR_)\b/i.test(text)) {
        violations.push(`${relative(root, path)}: removed Python Sidecar reference`);
      }
    }
  }
  for (const name of ["python", "pyproject.toml", "uv.lock"]) {
    if (existsSync(resolve(root, name))) violations.push(`${name}: Python runtime artifact remains`);
  }
}

function checkNames() {
  const pattern = /^[a-z][a-z0-9]*(?:[-_][a-z0-9]+)*$/;
  for (const directory of ["crates", "apps/desktop/src/features"]) {
    const path = resolve(root, directory);
    if (!existsSync(path)) continue;
    for (const entry of readdirSync(path, { withFileTypes: true })) {
      if (entry.isDirectory() && !pattern.test(entry.name)) {
        violations.push(`${directory}/${entry.name}: invalid module name`);
      }
    }
  }
}

function checkSchemas() {
  const schemaRoot = resolve(root, "contracts/schema");
  for (const path of walk(schemaRoot, (file) => file.endsWith(".schema.json"))) {
    try {
      const schema = JSON.parse(readFileSync(path, "utf8"));
      if (!schema.$id) violations.push(`${relative(root, path)}: missing $id`);
    } catch (error) {
      violations.push(`${relative(root, path)}: invalid JSON (${error.message})`);
    }
  }
}

function checkGeneratedContracts() {
  const script = resolve(root, "skills/opendesk/scripts/sync-contracts.mjs");
  const result = spawnSync(process.execPath, [script, "--check"], { cwd: root, encoding: "utf8" });
  if (result.status !== 0) {
    violations.push(`generated contracts are stale\n${(result.stderr || result.stdout).trim()}`);
  }
}

checkFrontendBoundary();
checkZeroPythonRuntime();
checkNames();
checkSchemas();
checkGeneratedContracts();

if (violations.length) {
  for (const violation of violations) console.error(`ERROR ${violation}`);
  process.exitCode = 1;
} else {
  console.log("architecture checks passed");
}

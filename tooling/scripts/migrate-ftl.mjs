#!/usr/bin/env node
/**
 * 一次性迁移脚本：把 `locales/{domain}/{zh-cn|en-us}.json` 转成 Fluent `.ftl`。
 *
 * 规则：
 * - 嵌套 JSON 扁平化为 message id：`a.b.c` → `a-b-c`。
 * - 占位符统一：`{param}` 与 `{{param}}` → `{$param}`。
 * - 输出 `locales/{domain}/{zh-CN|en-US}.ftl`。
 * - 自校验：JSON 叶子数 == FTL message 数。
 *
 * 用法：`node tooling/scripts/migrate-ftl.mjs [--check]`
 *   --check 只校验不写文件（CI 用）。
 *
 * @author coisini
 * @created 2026-08-06
 */

import { readdirSync, readFileSync, writeFileSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const localesDir = join(root, "apps/desktop/src/i18n/locales");
const checkOnly = process.argv.includes("--check");

/** 语言代码 → JSON 文件名标签。 */
const LOCALE_TAGS = {
  "zh-CN": "zh-cn",
  "en-US": "en-us",
} ;

function flatten(obj, prefix = "", out = {}) {
  for (const [key, value] of Object.entries(obj)) {
    const path = prefix ? `${prefix}.${key}` : key;
    if (value && typeof value === "object" && !Array.isArray(value)) {
      flatten(value, path, out);
    } else {
      out[path] = String(value);
    }
  }
  return out;
}

/** `a.b.c` → `a-b-c`。 */
function toMessageId(rest) {
  return rest.replaceAll(".", "-");
}

/** `{param}` / `{{param}}` → `{$param}`（FTL 变量需 `$` 前缀）。 */
function normalizePlaceholders(text) {
  return text
    .replace(/\{\{([\w]+)\}\}/g, (_, name) => `{$${name}}`)
    .replace(/\{([\w]+)\}/g, (_, name) => `{$${name}}`);
}

/**
 * 输出 FTL 值（裸值，不加引号）。
 *
 * 注意：JS 端 @fluent/bundle 不剥离引号，Rust 端 fluent-bundle 会剥离，
 * 为保证两端一致使用裸值。仅转义会破坏解析的反斜杠与换行。
 */
function toFtlValue(text) {
  return text.replaceAll("\\", "\\\\").replaceAll("\n", "\\n");
}

function renderFtl(messages) {
  const lines = [];
  for (const [path, text] of Object.entries(messages)) {
    const id = toMessageId(path);
    const value = toFtlValue(normalizePlaceholders(text));
    lines.push(`${id} = ${value}`);
  }
  return `${lines.join("\n")}\n`;
}

function countLeaves(obj) {
  let count = 0;
  for (const value of Object.values(obj)) {
    if (value && typeof value === "object" && !Array.isArray(value)) {
      count += countLeaves(value);
    } else {
      count += 1;
    }
  }
  return count;
}

let failures = 0;
for (const domain of readdirSync(localesDir, { withFileTypes: true }).filter((d) => d.isDirectory())) {
  for (const [localeCode, tag] of Object.entries(LOCALE_TAGS)) {
    const jsonPath = join(localesDir, domain.name, `${tag}.json`);
    if (!existsSync(jsonPath)) {
      continue;
    }
    const raw = JSON.parse(readFileSync(jsonPath, "utf-8"));
    const messages = flatten(raw);
    const ftl = renderFtl(messages);
    const leafCount = countLeaves(raw);

    if (Object.keys(messages).length !== leafCount) {
      console.error(`[migrate-ftl] ${domain.name}/${tag}: leaf mismatch`);
      failures += 1;
      continue;
    }

    const ftlPath = join(localesDir, domain.name, `${localeCode}.ftl`);
    if (checkOnly) {
      if (!existsSync(ftlPath)) {
        console.error(`[migrate-ftl] ${domain.name}/${localeCode}.ftl missing`);
        failures += 1;
      }
      continue;
    }
    writeFileSync(ftlPath, ftl, "utf-8");
    console.log(`[migrate-ftl] wrote ${domain.name}/${localeCode}.ftl (${Object.keys(messages).length} messages)`);
  }
}

if (checkOnly && failures > 0) {
  console.error(`[migrate-ftl] ${failures} failure(s)`);
  process.exit(1);
}

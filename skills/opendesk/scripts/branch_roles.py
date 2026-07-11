"""Branch role resolution and active Cursor rule generation."""

from __future__ import annotations

import json
import re
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CONFIG_PATH = ROOT / "skills" / "opendesk" / "config" / "branch_roles.json"
ACTIVE_RULE_PATH = ROOT / ".cursor" / "rules" / "active-branch.mdc"
RULES_DIR = ROOT / ".cursor" / "rules"

SLUG_RE = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")


@dataclass(frozen=True)
class RoleConfig:
    key: str
    label: str
    description: str
    branch_prefix: str
    allowed_globs: tuple[str, ...]
    optional_globs: tuple[str, ...]
    forbidden_globs: tuple[str, ...]
    rule_files: tuple[str, ...]


def load_config() -> dict:
    return json.loads(CONFIG_PATH.read_text(encoding="utf-8"))


def list_role_keys() -> list[str]:
    config = load_config()
    return [key for key in config["roles"] if key != "integration"]


def role_from_config(key: str) -> RoleConfig:
    config = load_config()
    if key not in config["roles"]:
        raise KeyError(f"unknown role: {key}")
    raw = config["roles"][key]
    return RoleConfig(
        key=key,
        label=raw["label"],
        description=raw["description"],
        branch_prefix=raw["branch_prefix"],
        allowed_globs=tuple(raw["allowed_globs"]),
        optional_globs=tuple(raw.get("optional_globs", [])),
        forbidden_globs=tuple(raw.get("forbidden_globs", [])),
        rule_files=tuple(raw.get("rule_files", [])),
    )


def validate_slug(slug: str) -> str:
    slug = slug.strip().lower()
    if not slug:
        raise ValueError("slug is required (e.g. m5-layout)")
    if not SLUG_RE.fullmatch(slug):
        raise ValueError(f"invalid slug: {slug!r} (use kebab-case, e.g. m5-layout)")
    return slug


def branch_name_for(role_key: str, slug: str | None = None) -> str:
    role = role_from_config(role_key)
    if slug:
        return f"{role.branch_prefix}/{validate_slug(slug)}"
    return f"role/{role.branch_prefix}"


def resolve_role(branch: str) -> RoleConfig:
    config = load_config()
    legacy = config.get("legacy_branch_map", {})
    if branch in legacy:
        return role_from_config(legacy[branch])
    if branch == "main":
        return role_from_config("integration")

    if branch.startswith("role/"):
        suffix = branch.removeprefix("role/")
        if suffix in config["roles"]:
            return role_from_config(suffix)

    prefix = branch.split("/", 1)[0]
    if prefix in config["roles"]:
        return role_from_config(prefix)

    raise ValueError(
        f"cannot infer role from branch {branch!r}; "
        f"use frontend/<slug>, python/<slug>, contract/<slug>, role/<role>, or main"
    )


def render_active_rule(branch: str, role: RoleConfig) -> str:
    allowed = "\n".join(f"- `{g}`" for g in role.allowed_globs)
    optional = (
        "\n".join(f"- `{g}`" for g in role.optional_globs)
        if role.optional_globs
        else "- _(无 — 非本分支职责)_"
    )
    forbidden = (
        "\n".join(f"- `{g}`" for g in role.forbidden_globs)
        if role.forbidden_globs
        else "- _(无额外禁止路径)_"
    )
    rules = (
        "\n".join(f"- [`.cursor/rules/{name}`]({name})" for name in role.rule_files)
        if role.rule_files
        else "- [`master.md`](master.md) only"
    )

    return f"""---
description: Active branch scope for {branch} (auto-generated — run pnpm branch:sync)
alwaysApply: true
---

# 当前分支：`{branch}`

**角色：** {role.label}

{role.description}

## 允许修改

{allowed}

## 可选（跨端契约 — 先改 Contract 再 codegen）

{optional}

## 禁止修改（除非用户明确要求扩 scope）

{forbidden}

## 细则规则

{rules}

---

> 由 `skills/opendesk/scripts/sync_branch_rules.py` 根据分支名生成。
> 切换分支后若约束不符，运行 `pnpm branch:sync`。
"""


def sync_active_rule(*, branch: str | None = None, dry_run: bool = False) -> Path:
    if branch is None:
        import subprocess

        result = subprocess.run(
            ["git", "branch", "--show-current"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=True,
        )
        branch = result.stdout.strip()
        if not branch:
            raise RuntimeError("detached HEAD — checkout a branch first")

    role = resolve_role(branch)
    content = render_active_rule(branch, role)
    if dry_run:
        return ACTIVE_RULE_PATH

    ACTIVE_RULE_PATH.parent.mkdir(parents=True, exist_ok=True)
    ACTIVE_RULE_PATH.write_text(content, encoding="utf-8", newline="\n")
    return ACTIVE_RULE_PATH

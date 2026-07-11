#!/usr/bin/env python3
"""Create a role-scoped git branch and sync Cursor rules."""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "skills" / "opendesk" / "scripts"))

from _common import setup_logging  # noqa: E402
from branch_roles import (  # noqa: E402
    branch_name_for,
    list_role_keys,
    role_from_config,
    sync_active_rule,
    validate_slug,
)


def _run(cmd: list[str], *, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=ROOT, text=True, check=check)


def _current_branch() -> str:
    return _run(["git", "branch", "--show-current"]).stdout.strip()


def _branch_exists(name: str) -> bool:
    result = _run(["git", "rev-parse", "--verify", name], check=False)
    return result.returncode == 0


def _prompt_role() -> str:
    keys = list_role_keys()
    print("\nOpenDesk — 创建角色分支\n")
    for index, key in enumerate(keys, start=1):
        role = role_from_config(key)
        print(f"  {index}) {key:<10} — {role.label}")
    print()
    while True:
        choice = input("选择角色编号或名称: ").strip().lower()
        if choice.isdigit():
            idx = int(choice) - 1
            if 0 <= idx < len(keys):
                return keys[idx]
        if choice in keys:
            return choice
        print("无效选择，请重试。")


def _prompt_slug(role_key: str) -> str:
    while True:
        slug = input(
            f"任务 slug（kebab-case，如 m5-layout；回车则用 role/{role_key} 长期分支）: "
        ).strip()
        if not slug:
            return ""
        try:
            validate_slug(slug)
            return slug
        except ValueError as exc:
            print(exc)


def _prompt_base(default: str = "main") -> str:
    base = input(f"基于分支 [{default}]: ").strip() or default
    if not _branch_exists(base):
        remote = f"origin/{base}"
        if _branch_exists(remote):
            print(f"本地无 {base}，将从 {remote} 创建跟踪分支…")
            _run(["git", "fetch", "origin", base])
            _run(["git", "branch", base, remote])
        else:
            raise RuntimeError(f"base branch not found: {base}")
    return base


def create_branch(
    role_key: str,
    slug: str | None,
    *,
    base: str = "main",
    checkout: bool = True,
) -> str:
    name = branch_name_for(role_key, slug or None)
    if _branch_exists(name):
        raise RuntimeError(f"branch already exists: {name}")

    _run(["git", "fetch", "origin", base], check=False)
    if not _branch_exists(base):
        remote = f"origin/{base}"
        if _branch_exists(remote):
            _run(["git", "branch", base, remote])
        else:
            raise RuntimeError(f"base branch not found: {base}")

    _run(["git", "branch", name, base])
    if checkout:
        _run(["git", "checkout", name])
        sync_active_rule(branch=name)
    return name


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Create a role-scoped branch (frontend/*, python/*, contract/*).",
        epilog=(
            "Examples:\n"
            "  python create_branch.py frontend m5-layout\n"
            "  python create_branch.py python sidecar-logging\n"
            "  python create_branch.py --interactive\n"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("role", nargs="?", help="frontend | python | contract")
    parser.add_argument("slug", nargs="?", help="kebab-case task slug, e.g. m5-layout")
    parser.add_argument("-i", "--interactive", action="store_true", help="prompt for inputs")
    parser.add_argument("--base", default="main", help="base branch (default: main)")
    parser.add_argument("--no-checkout", action="store_true")
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    setup_logging(args.verbose)

    try:
        if args.interactive or not args.role:
            role_key = _prompt_role()
            slug = _prompt_slug(role_key)
            base = _prompt_base(args.base)
        else:
            role_key = args.role.strip().lower()
            if role_key not in list_role_keys():
                choices = ", ".join(list_role_keys())
                raise ValueError(f"unknown role {role_key!r}; choose: {choices}")
            slug = args.slug.strip() if args.slug else None
            base = args.base

        name = create_branch(
            role_key,
            slug,
            base=base,
            checkout=not args.no_checkout,
        )
        role = role_from_config(role_key)
        print(f"\nOK Created branch `{name}` ({role.label})")
        if not args.no_checkout:
            print("OK Synced rules -> .cursor/rules/active-branch.mdc")
            print(f"\n当前分支: {_current_branch()}")
        return 0
    except (RuntimeError, ValueError, subprocess.CalledProcessError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())

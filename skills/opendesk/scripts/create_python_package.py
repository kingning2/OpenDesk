#!/usr/bin/env python3
"""Scaffold a Python package and register it in the uv workspace."""

from __future__ import annotations

import argparse
import logging

from _common import (
    PYPROJECT_TOML,
    PYTHON_PACKAGES,
    ensure_ruff_first_party,
    ensure_uv_workspace_member,
    setup_logging,
    validate_name,
    write_text,
)


def _deps_block(dependencies: list[str]) -> str:
    if not dependencies:
        return "dependencies = []"
    lines = ",\n".join(f'  "{dep}"' for dep in dependencies)
    return f"dependencies = [\n{lines},\n]"


def _pyproject_content(name: str, dependencies: list[str]) -> str:
    return f"""[project]
name = "{name}"
version = "0.1.0"
requires-python = ">=3.13"
{_deps_block(dependencies)}

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[tool.hatch.build.targets.wheel]
packages = ["src/{name}"]
"""


def _workspace_path(name: str) -> str:
    return f"python/packages/{name}"


def _uv_dep(name: str) -> str:
    return f"{name}>=0.1.0"


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__,
        epilog="Example: python create_python_package.py --name embed",
    )
    parser.add_argument("--name", required=True, help="package name (snake_case)")
    parser.add_argument(
        "--dep",
        action="append",
        default=[],
        metavar="PKG",
        help="workspace dependency (e.g. contracts, shared); repeatable",
    )
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    setup_logging(args.verbose)

    try:
        name = validate_name(args.name)
    except ValueError as exc:
        logging.error("%s", exc)
        return 1

    pkg_dir = PYTHON_PACKAGES / name
    if pkg_dir.exists() and not args.dry_run:
        logging.error("package directory already exists: %s", pkg_dir)
        return 1

    deps = sorted({_uv_dep(validate_name(dep)) for dep in args.dep})
    workspace_member = _workspace_path(name)

    write_text(pkg_dir / "pyproject.toml", _pyproject_content(name, deps), dry_run=args.dry_run)
    write_text(
        pkg_dir / "src" / name / "__init__.py",
        f'"""{name} package scaffold."""\n',
        dry_run=args.dry_run,
    )
    write_text(
        pkg_dir / "tests" / "test_scaffold.py",
        f'def test_import_{name}() -> None:\n    import {name}\n\n    assert {name} is not None\n',
        dry_run=args.dry_run,
    )
    write_text(pkg_dir / "README.md", f"# {name}\n\nPython package scaffold. No business logic.\n", dry_run=args.dry_run)

    ensure_uv_workspace_member(workspace_member, dry_run=args.dry_run)
    ensure_ruff_first_party(name, dry_run=args.dry_run)

    logging.info("python package %r scaffold complete", name)
    logging.info("registered in %s [tool.uv.workspace].members", PYPROJECT_TOML)
    logging.info("next: uv sync (if using uv) and pnpm lint:python")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

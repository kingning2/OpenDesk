#!/usr/bin/env python3
"""Check naming conventions for crates and features."""

from __future__ import annotations

import argparse
import logging
import re

from _common import CRATES, FORBIDDEN_SUFFIXES, ROOT, setup_logging

FEATURES_DIR = ROOT / "apps" / "desktop" / "src" / "features"


def check_name(name: str, context: str, violations: list[str]) -> None:
    if not re.fullmatch(r"[a-z][a-z0-9]*(-[a-z0-9]+)*", name.replace("_", "-")):
        violations.append(f"{context}: invalid name '{name}'")
    for suffix in FORBIDDEN_SUFFIXES:
        if name.lower().endswith(suffix.lower()):
            violations.append(f"{context}: forbidden suffix '{suffix}' in '{name}'")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    setup_logging(args.verbose)

    violations: list[str] = []
    if CRATES.exists():
        for path in CRATES.iterdir():
            if path.is_dir():
                check_name(path.name, f"crate {path.name}", violations)

    if FEATURES_DIR.exists():
        for path in FEATURES_DIR.iterdir():
            if path.is_dir():
                check_name(path.name, f"feature {path.name}", violations)

    for v in violations:
        logging.error(v)
    return 1 if violations else 0


if __name__ == "__main__":
    raise SystemExit(main())

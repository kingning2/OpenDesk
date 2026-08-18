#!/usr/bin/env python3
"""Check Python code does not access SQLite."""

from __future__ import annotations

import argparse
import logging
import re

from _common import ROOT, setup_logging

PYTHON_ROOT = ROOT / "python"
SQLITE_PATTERNS = [
    re.compile(r"\bimport\s+sqlite3\b"),
    re.compile(r"\bfrom\s+sqlite3\b"),
    re.compile(r"\bsqlalchemy\b"),
    re.compile(r"\baiosqlite\b"),
]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    setup_logging(args.verbose)

    violations: list[str] = []
    for path in PYTHON_ROOT.rglob("*.py"):
        if "skills" in path.parts:
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        for pat in SQLITE_PATTERNS:
            if pat.search(text):
                violations.append(f"{path}: possible SQLite access ({pat.pattern})")

    for v in violations:
        logging.error(v)
    return 1 if violations else 0


if __name__ == "__main__":
    raise SystemExit(main())

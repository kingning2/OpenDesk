#!/usr/bin/env python3
"""Generate a filtered project tree for documentation."""

from __future__ import annotations

import argparse
import logging
from pathlib import Path

from _common import ROOT, setup_logging

SKIP_DIRS = {
    "node_modules",
    ".git",
    "target",
    "dist",
    ".venv",
    "__pycache__",
    ".cursor",
}


def tree(path: Path, prefix: str = "", max_depth: int = 4, depth: int = 0) -> list[str]:
    lines: list[str] = []
    if depth > max_depth:
        return lines
    try:
        entries = sorted(path.iterdir(), key=lambda p: (p.is_file(), p.name.lower()))
    except PermissionError:
        return lines
    entries = [e for e in entries if e.name not in SKIP_DIRS]
    for i, entry in enumerate(entries):
        connector = "└── " if i == len(entries) - 1 else "├── "
        lines.append(f"{prefix}{connector}{entry.name}{'/' if entry.is_dir() else ''}")
        if entry.is_dir():
            extension = "    " if i == len(entries) - 1 else "│   "
            lines.extend(tree(entry, prefix + extension, max_depth, depth + 1))
    return lines


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--max-depth", type=int, default=3)
    parser.add_argument("-o", "--output", type=Path, help="write to file")
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    setup_logging(args.verbose)

    header = f"{ROOT.name}/\n"
    body = "\n".join(tree(ROOT, max_depth=args.max_depth))
    result = header + body + "\n"

    if args.output:
        args.output.write_text(result, encoding="utf-8")
        logging.info("wrote %s", args.output)
    else:
        print(result)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

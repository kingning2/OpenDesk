#!/usr/bin/env python3
"""Check React feature files do not import @tauri-apps/api directly."""

from __future__ import annotations

import argparse
import logging
import re
import sys

from _common import ROOT, setup_logging

FEATURES_DIR = ROOT / "apps" / "desktop" / "src" / "features"
TAURI_IMPORT = re.compile(r"""from\s+['"]@tauri-apps/api""")
INVOKE = re.compile(r"""\binvoke\s*\(""")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    setup_logging(args.verbose)

    violations: list[str] = []
    if FEATURES_DIR.exists():
        for path in FEATURES_DIR.rglob("*"):
            if path.suffix not in (".ts", ".tsx"):
                continue
            text = path.read_text(encoding="utf-8", errors="replace")
            if TAURI_IMPORT.search(text):
                violations.append(f"{path}: direct @tauri-apps/api import")
            if INVOKE.search(text):
                violations.append(f"{path}: direct invoke() call")

    for v in violations:
        logging.error(v)

    return 1 if violations else 0


if __name__ == "__main__":
    raise SystemExit(main())

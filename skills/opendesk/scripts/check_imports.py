#!/usr/bin/env python3
"""Check feature crates do not depend on each other."""

from __future__ import annotations

import argparse
import logging
import re

from _common import CRATES, FEATURES, setup_logging

DEP_RE = re.compile(r"^(\w+)\s*=\s*\{\s*path\s*=\s*\"\.\./(\w+)\"", re.MULTILINE)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    setup_logging(args.verbose)

    violations: list[str] = []
    for feature in FEATURES:
        cargo = CRATES / feature / "Cargo.toml"
        if not cargo.exists():
            continue
        text = cargo.read_text(encoding="utf-8")
        for _name, dep in DEP_RE.findall(text):
            if dep in FEATURES and dep != feature:
                violations.append(f"{cargo}: feature crate depends on feature '{dep}'")

    for v in violations:
        logging.error(v)
    return 1 if violations else 0


if __name__ == "__main__":
    raise SystemExit(main())

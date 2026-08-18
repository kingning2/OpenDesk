#!/usr/bin/env python3
"""Validate contract schema files exist and are valid JSON."""

from __future__ import annotations

import argparse
import json
import logging

from _common import CONTRACTS, setup_logging


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    setup_logging(args.verbose)

    schema_root = CONTRACTS / "schema"
    if not schema_root.exists():
        logging.warning("no contracts/schema directory")
        return 0

    errors: list[str] = []
    for path in schema_root.rglob("*.schema.json"):
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            errors.append(f"{path}: invalid JSON: {exc}")
            continue
        if "$id" not in data:
            errors.append(f"{path}: missing $id")
        if data.get("type") == "object" and "additionalProperties" not in data:
            logging.warning("%s: recommend additionalProperties: false", path)

    for e in errors:
        logging.error(e)
    logging.info("checked %d schema files", len(list(schema_root.rglob("*.schema.json"))))
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())

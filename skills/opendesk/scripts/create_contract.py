#!/usr/bin/env python3
"""Create a JSON Schema contract file."""

from __future__ import annotations

import argparse
import json
import logging

from _common import CONTRACTS, setup_logging, validate_name, write_text

KINDS = ("dto", "ipc", "event", "error")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--feature", required=True)
    parser.add_argument("--name", required=True, help="schema base name (kebab-case)")
    parser.add_argument("--kind", choices=KINDS, default="dto")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    setup_logging(args.verbose)

    try:
        feature = validate_name(args.feature)
    except ValueError as exc:
        logging.error("%s", exc)
        return 1

    name = args.name.strip().lower()
    schema = {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": f"opendesk://{feature}/{args.kind}/{name}/v1",
        "title": f"{feature}/{name}",
        "type": "object",
        "required": [],
        "properties": {},
        "additionalProperties": False,
    }
    path = CONTRACTS / "schema" / "v1" / feature / args.kind / f"{name}.schema.json"
    write_text(path, json.dumps(schema, indent=2) + "\n", dry_run=args.dry_run)
    logging.info("contract schema: %s", path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Create IPC request/response contract pair."""

from __future__ import annotations

import argparse
import json
import logging

from _common import CONTRACTS, setup_logging, validate_name, write_text


def _schema(feature: str, kind: str, name: str) -> dict:
    return {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": f"opendesk://{feature}/ipc/{name}.{kind}/v1",
        "title": f"{feature}/{name}.{kind}",
        "type": "object",
        "required": [],
        "properties": {},
        "additionalProperties": False,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--feature", required=True)
    parser.add_argument("--command", required=True, help="e.g. list_threads")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    setup_logging(args.verbose)

    try:
        feature = validate_name(args.feature)
    except ValueError as exc:
        logging.error("%s", exc)
        return 1

    cmd = args.command.strip().lower().replace("-", "_")
    base = CONTRACTS / "schema" / "v1" / feature / "ipc"
    write_text(
        base / f"{cmd}.request.schema.json",
        json.dumps(_schema(feature, "request", cmd), indent=2) + "\n",
        dry_run=args.dry_run,
    )
    write_text(
        base / f"{cmd}.response.schema.json",
        json.dumps(_schema(feature, "response", cmd), indent=2) + "\n",
        dry_run=args.dry_run,
    )
    logging.info("ipc contracts for %s_%s", feature, cmd)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

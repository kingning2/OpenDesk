#!/usr/bin/env python3
"""Create an event contract schema."""

from __future__ import annotations

import argparse
import logging

from create_contract import main as create_contract_main


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--feature", required=True)
    parser.add_argument("--name", required=True, help="e.g. message.sent")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("-v", "--verbose", action="store_true")
    args, _ = parser.parse_known_args()
    import sys

    sys.argv = [
        "create_contract.py",
        "--feature",
        args.feature,
        "--name",
        args.name.replace(".", "-"),
        "--kind",
        "event",
    ]
    if args.dry_run:
        sys.argv.append("--dry-run")
    if args.verbose:
        sys.argv.append("-v")
    return create_contract_main()


if __name__ == "__main__":
    raise SystemExit(main())

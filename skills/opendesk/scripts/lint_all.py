#!/usr/bin/env python3
"""Run full project lint (pnpm lint)."""

from __future__ import annotations

import argparse
import logging
import subprocess
import sys

from _common import ROOT, setup_logging


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    setup_logging(args.verbose)

    logging.info("running pnpm lint in %s", ROOT)
    try:
        result = subprocess.run(
            ["pnpm", "lint"],
            cwd=ROOT,
            check=False,
        )
    except FileNotFoundError:
        logging.error("pnpm not found in PATH")
        return 1

    if result.returncode != 0:
        logging.error("lint failed with code %d", result.returncode)
    else:
        logging.info("lint passed")
    return result.returncode


if __name__ == "__main__":
    raise SystemExit(main())

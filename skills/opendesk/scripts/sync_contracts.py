#!/usr/bin/env python3
"""Sync contracts to codegen output directories (skeleton)."""

from __future__ import annotations

import argparse
import logging
from pathlib import Path

from _common import CONTRACTS, ROOT, setup_logging, write_text

TS_OUT = ROOT / "packages" / "contracts" / "src" / "generated"
PY_OUT = ROOT / "python" / "packages" / "contracts" / "src" / "contracts" / "generated"
RS_OUT = ROOT / "crates" / "common" / "src" / "contracts"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dry-run", action="store_true", help="show actions only")
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    setup_logging(args.verbose)

    schema_root = CONTRACTS / "schema" / "v1"
    if not schema_root.exists():
        logging.warning("no schemas at %s", schema_root)
        return 0

    schemas = list(schema_root.rglob("*.schema.json"))
    logging.info("found %d schema files", len(schemas))

    index_ts = "// Auto-generated contract index (skeleton).\n// TODO: wire contracts/codegen pipeline.\n"
    index_py = '"""Auto-generated contract index (skeleton)."""\n'
    index_rs = "// Auto-generated contract index (skeleton).\n"

    for path in schemas:
        rel = path.relative_to(schema_root)
        logging.debug("would codegen %s", rel)

    write_text(TS_OUT / "index.ts", index_ts, dry_run=args.dry_run)
    write_text(PY_OUT / "__init__.py", index_py, dry_run=args.dry_run)
    write_text(RS_OUT / "mod.rs", index_rs, dry_run=args.dry_run)

    logging.info("sync complete (skeleton placeholders)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

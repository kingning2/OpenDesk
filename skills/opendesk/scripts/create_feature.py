#!/usr/bin/env python3
"""Scaffold a full OpenDesk feature (contract + src-tauri UseCase + frontend)."""

from __future__ import annotations

import argparse
import logging

from _common import (
    CONTRACTS,
    DESKTOP_FEATURES,
    SRC_TAURI,
    setup_logging,
    validate_name,
    write_text,
)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--name", required=True, help="feature name (e.g. chat)")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    setup_logging(args.verbose)

    try:
        name = validate_name(args.name)
    except ValueError as exc:
        logging.error("%s", exc)
        return 1

    for sub in ("dto", "ipc", "event", "error"):
        write_text(CONTRACTS / "schema" / "v1" / name / sub / ".gitkeep", "", dry_run=args.dry_run)

    usecase = SRC_TAURI / f"{name}.rs"
    write_text(
        usecase,
        f"""//! {name} UseCase 骨架（业务代码放在 src-tauri，不放 crates/）。

// TODO: 实现 UseCase 并注册 Tauri command。
""",
        dry_run=args.dry_run,
    )

    fe_dir = DESKTOP_FEATURES / name
    write_text(
        fe_dir / "index.ts",
        f'export const {name}Feature = {{\n  id: "{name}",\n}};\n',
        dry_run=args.dry_run,
    )
    for sub in ("pages", "components", "hooks"):
        write_text(fe_dir / sub / ".gitkeep", "", dry_run=args.dry_run)

    logging.info("feature %r scaffold complete", name)
    logging.info("next: edit contracts, run sync_contracts.py, register tauri commands")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

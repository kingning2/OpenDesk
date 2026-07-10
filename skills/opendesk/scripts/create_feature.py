#!/usr/bin/env python3
"""Scaffold a full OpenDesk feature (contract + crate + frontend)."""

from __future__ import annotations

import argparse
import logging
import sys

from _common import (
    CONTRACTS,
    CRATES,
    DESKTOP_FEATURES,
    ensure_workspace_member,
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

    crate_dir = CRATES / name
    write_text(
        crate_dir / "Cargo.toml",
        f"""[package]
name = "{name}"
version.workspace = true
edition.workspace = true

[dependencies]
common = {{ path = "../common" }}
kernel = {{ path = "../kernel" }}
ports = {{ path = "../ports" }}
serde = {{ workspace = true }}
thiserror = "2"
tracing = "0.1"
""",
        dry_run=args.dry_run,
    )
    write_text(
        crate_dir / "src" / "lib.rs",
        f"//! {name} crate scaffold.\n\npub mod app;\npub mod domain;\n",
        dry_run=args.dry_run,
    )
    write_text(crate_dir / "src" / "app" / "mod.rs", "", dry_run=args.dry_run)
    write_text(crate_dir / "src" / "domain" / "mod.rs", "", dry_run=args.dry_run)
    write_text(
        crate_dir / "tests" / "scaffold_test.rs",
        "#[test]\nfn crate_links() {\n    assert!(true);\n}\n",
        dry_run=args.dry_run,
    )
    ensure_workspace_member(name, dry_run=args.dry_run)

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

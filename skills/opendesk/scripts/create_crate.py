#!/usr/bin/env python3
"""Scaffold a Rust crate and register it in the workspace."""

from __future__ import annotations

import argparse
import logging

from _common import CRATES, ensure_workspace_member, setup_logging, validate_name, write_text


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--name", required=True)
    parser.add_argument("--kind", choices=("feature", "infra"), default="feature")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    setup_logging(args.verbose)

    try:
        name = validate_name(args.name)
    except ValueError as exc:
        logging.error("%s", exc)
        return 1

    crate_dir = CRATES / name
    deps = """common = { path = "../common" }
serde = { workspace = true }
"""
    lib_body = f"//! {name} crate scaffold.\n"
    if args.kind == "feature":
        deps += """kernel = { path = "../kernel" }
ports = { path = "../ports" }
thiserror = "2"
tracing = "0.1"
"""
        lib_body += "\npub mod app;\npub mod domain;\n"
        write_text(crate_dir / "src" / "app" / "mod.rs", "", dry_run=args.dry_run)
        write_text(crate_dir / "src" / "domain" / "mod.rs", "", dry_run=args.dry_run)

    write_text(
        crate_dir / "Cargo.toml",
        f"""[package]
name = "{name}"
version.workspace = true
edition.workspace = true

[dependencies]
{deps}""",
        dry_run=args.dry_run,
    )
    write_text(crate_dir / "src" / "lib.rs", lib_body, dry_run=args.dry_run)
    write_text(
        crate_dir / "tests" / "scaffold_test.rs",
        "#[test]\nfn crate_links() {\n    assert!(true);\n}\n",
        dry_run=args.dry_run,
    )
    ensure_workspace_member(name, dry_run=args.dry_run)
    logging.info("crate %r (%s) created", name, args.kind)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

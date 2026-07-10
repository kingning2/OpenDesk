#!/usr/bin/env python3
"""Create a Query Port trait in crates/ports."""

from __future__ import annotations

import argparse
import logging
import re

from _common import CRATES, read_text, setup_logging, validate_name, write_text

PORTS_LIB = CRATES / "ports" / "src" / "lib.rs"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--name", required=True, help="e.g. chat_query")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    setup_logging(args.verbose)

    try:
        name = validate_name(args.name)
    except ValueError as exc:
        logging.error("%s", exc)
        return 1

    trait_name = "".join(part.capitalize() for part in name.split("_")) + "Port"
    mod_file = CRATES / "ports" / "src" / f"{name}.rs"
    write_text(
        mod_file,
        f"""//! Query port scaffold for `{name}`.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum {trait_name.replace("Port", "Error")} {{
    #[error("not found")]
    NotFound,
}}

pub trait {trait_name}: Send + Sync {{
    // TODO: add read-only query methods
}}
""",
        dry_run=args.dry_run,
    )

    if PORTS_LIB.exists():
        content = read_text(PORTS_LIB)
        mod_line = f"pub mod {name};"
        if mod_line not in content:
            if not content.endswith("\n"):
                content += "\n"
            content += f"\n{mod_line}\n"
            write_text(PORTS_LIB, content, dry_run=args.dry_run)

    logging.info("query port %s in crates/ports", trait_name)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

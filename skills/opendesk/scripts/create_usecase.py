#!/usr/bin/env python3
"""Create a UseCase module skeleton in a feature crate."""

from __future__ import annotations

import argparse
import logging

from _common import CRATES, read_text, setup_logging, validate_name, write_text


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--crate", required=True, help="feature crate name")
    parser.add_argument("--name", required=True, help="usecase module name")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    setup_logging(args.verbose)

    try:
        crate = validate_name(args.crate)
        usecase = validate_name(args.name)
    except ValueError as exc:
        logging.error("%s", exc)
        return 1

    app_mod = CRATES / crate / "src" / "app" / "mod.rs"
    usecase_file = CRATES / crate / "src" / "app" / f"{usecase}.rs"
    struct_name = "".join(p.capitalize() for p in usecase.split("_"))

    write_text(
        usecase_file,
        f"""//! UseCase scaffold: {usecase}

use tracing::instrument;

pub struct {struct_name};

impl {struct_name} {{
    #[instrument]
    pub fn execute(&self) -> Result<(), super::super::domain::DomainError> {{
        // TODO: inject ports via constructor
        Ok(())
    }}
}}
""",
        dry_run=args.dry_run,
    )

    if app_mod.exists():
        content = read_text(app_mod)
        mod_line = f"pub mod {usecase};"
        if mod_line not in content:
            content = content + ("\n" if content else "") + mod_line + "\n"
            write_text(app_mod, content, dry_run=args.dry_run)

    logging.info("usecase %s in crate %s", usecase, crate)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

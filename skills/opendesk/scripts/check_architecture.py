#!/usr/bin/env python3
"""Aggregate architecture compliance checks for OpenDesk."""

from __future__ import annotations

import argparse
import importlib.util
import logging
import re
import sys
from pathlib import Path

from _common import ROOT, setup_logging

SCRIPTS_DIR = Path(__file__).resolve().parent

# React → Python heuristics
REACT_ROOT = ROOT / "apps" / "desktop" / "src"
PYTHON_URL = re.compile(r"https?://(?:127\.0\.0\.1|localhost):\d+")
WEBSOCKET = re.compile(r"new\s+WebSocket\s*\(")

# Rust unwrap heuristic
UNWRAP = re.compile(r"\.unwrap\(\)|\.expect\(|panic!\(")


def _load_check(name: str) -> int:
    path = SCRIPTS_DIR / f"{name}.py"
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        logging.error("cannot load %s", path)
        return 1
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return int(module.main())


def check_react_python() -> list[str]:
    violations: list[str] = []
    if not REACT_ROOT.exists():
        return violations
    for path in REACT_ROOT.rglob("*"):
        if path.suffix not in (".ts", ".tsx"):
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        if PYTHON_URL.search(text):
            violations.append(f"{path}: possible direct localhost HTTP to Python")
        if WEBSOCKET.search(text):
            violations.append(f"{path}: possible WebSocket bypass")
    return violations


def check_rust_unwrap() -> list[str]:
    violations: list[str] = []
    for root in (ROOT / "crates", ROOT / "apps" / "desktop" / "src-tauri"):
        if not root.exists():
            continue
        for path in root.rglob("*.rs"):
            if "tests" in path.parts:
                continue
            text = path.read_text(encoding="utf-8", errors="replace")
            if UNWRAP.search(text):
                violations.append(f"{path}: unwrap/expect/panic detected")
    return violations


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("-v", "--verbose", action="store_true")
    args = parser.parse_args()
    setup_logging(args.verbose)

    exit_code = 0

    logging.info("=== check_layers (React → Tauri) ===")
    exit_code |= _load_check("check_layers")

    logging.info("=== check_boundary (Python → SQLite) ===")
    exit_code |= _load_check("check_boundary")

    logging.info("=== check_imports (Feature → Feature) ===")
    exit_code |= _load_check("check_imports")

    logging.info("=== check_naming ===")
    exit_code |= _load_check("check_naming")

    logging.info("=== check_contracts ===")
    exit_code |= _load_check("check_contracts")

    logging.info("=== inline: React → Python ===")
    for v in check_react_python():
        logging.error(v)
        exit_code = 1

    logging.info("=== inline: Rust unwrap/expect/panic ===")
    for v in check_rust_unwrap():
        logging.error(v)
        exit_code = 1

    if exit_code == 0:
        logging.info("architecture checks passed")
    else:
        logging.error("architecture checks failed")
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())

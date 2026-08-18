#!/usr/bin/env python3
"""Fix store methods: append ? to self.db calls used as return values."""

from pathlib import Path

path = Path("business/src/xianyu/stores.rs")
lines = path.read_text(encoding="utf-8").splitlines()
out = []
for i, line in enumerate(lines):
    stripped = line.rstrip()
    if (
        stripped.startswith("self.db.")
        and not stripped.endswith(";")
        and not stripped.endswith("?")
    ):
        nxt = lines[i + 1] if i + 1 < len(lines) else "}"
        if nxt.strip().startswith("}"):
            stripped += "?"
    out.append(stripped)
path.write_text("\n".join(out) + "\n", encoding="utf-8")

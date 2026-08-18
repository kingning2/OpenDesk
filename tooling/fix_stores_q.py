#!/usr/bin/env python3
from pathlib import Path

path = Path("business/src/xianyu/stores.rs")
lines = path.read_text(encoding="utf-8").splitlines()
out = []
for i, line in enumerate(lines):
    stripped = line.rstrip()
    if (
        stripped.lstrip().startswith("self.db.")
        and not stripped.endswith(";")
        and not stripped.endswith("?")
    ):
        j = i + 1
        while j < len(lines) and lines[j].strip() == "":
            j += 1
        if j < len(lines) and lines[j].strip() == "}":
            stripped = stripped + "?"
    out.append(stripped)
path.write_text("\n".join(out) + "\n", encoding="utf-8")

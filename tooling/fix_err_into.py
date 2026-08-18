#!/usr/bin/env python3
"""Add .into() to Err(...) when migrating to DingDaResult."""

import re
from pathlib import Path

ROOTS = [
    Path("crates/app/src"),
    Path("business/src"),
]


def fix_simple(content: str) -> str:
    # return Err("literal".to_string());
    content = content.replace(".to_string());", ".to_string().into());")
    # return Err(format!(...));
    content = re.sub(
        r"(return Err\(format!\([^)]*(?:\([^)]*\)[^)]*)*\))\);",
        r"\1.into());",
        content,
    )
    content = re.sub(
        r"(Err\(format!\([^)]*(?:\([^)]*\)[^)]*)*\))\)(?!\.into)",
        r"\1.into())",
        content,
    )
    # Err("plain")
    content = re.sub(r'Err\("([^"]*)"\)(?!\.into)', r'Err("\1".into())', content)
    return content


for root in ROOTS:
    base = Path(__file__).resolve().parents[1] / root
    for path in base.rglob("*.rs"):
        old = path.read_text(encoding="utf-8")
        new = fix_simple(old)
        if new != old:
            path.write_text(new, encoding="utf-8")
            print(path.relative_to(base.parent.parent if "crates" in str(root) else base.parent))

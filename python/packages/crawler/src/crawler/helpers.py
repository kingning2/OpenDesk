"""Shared crawl helpers (email extract, event ids).

Author: Xiaoman
Created: 2026-07-16
"""

from __future__ import annotations

import re
import uuid
from datetime import UTC, datetime


def now_iso() -> str:
    """Return UTC ISO-8601 timestamp.

    Returns:
        Timestamp string with ``Z`` suffix.
    """
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def new_event_id() -> str:
    """Allocate a unique event id.

    Returns:
        UUID string.
    """
    return str(uuid.uuid4())


def normalize_for_email(text: str) -> str:
    """Normalize obfuscated email markers in channel descriptions.

    Args:
        text: Raw description.

    Returns:
        Normalized text for regex matching.
    """
    result = text or ""
    result = re.sub(r"\[at\]|\(at\)|\s+at\s+", "@", result, flags=re.IGNORECASE)
    result = re.sub(r"\[dot\]|\(dot\)|\s+dot\s+", ".", result, flags=re.IGNORECASE)
    result = result.replace("（at）", "@").replace("（dot）", ".")
    result = result.replace("＠", "@").replace("。", ".")
    result = re.sub(r"\s*@\s*", "@", result)
    result = re.sub(r"\s*\.\s*", ".", result)
    return result


def extract_email(description: str | None) -> str:
    """Extract the first email-like address from a description.

    Args:
        description: Channel description text.

    Returns:
        Email string or empty string when not found.
    """
    if not description:
        return ""
    normalized = normalize_for_email(description)
    match = re.search(
        r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}",
        normalized,
    )
    return match.group(0) if match else ""

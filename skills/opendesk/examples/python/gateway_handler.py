"""Example: Gateway handler — no SQLite, no React."""

from __future__ import annotations

import logging

logger = logging.getLogger("opendesk.gateway.example")


def handle_ping(trace_id: str) -> dict[str, str]:
    logger.info("ping", extra={"trace_id": trace_id})
    return {"status": "ok"}

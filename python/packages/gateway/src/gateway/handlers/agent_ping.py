"""Sidecar handler: /v1/agent/ping (POST) — Python ← Rust only."""

from __future__ import annotations

import logging
from typing import Any

logger = logging.getLogger("opendesk.sidecar.agent")


def handle_agent_ping(payload: dict[str, Any] | None, *, trace_id: str) -> dict[str, Any]:
    """Contract: contracts/schema/v1/agent/sidecar/ping.*.schema.json"""
    logger.info("handle_agent_ping", extra={"trace_id": trace_id, "feature": "agent"})
    _ = payload
    return {"ok": True, "trace_id": trace_id}

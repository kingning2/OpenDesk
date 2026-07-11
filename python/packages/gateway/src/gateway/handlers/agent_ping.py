"""Sidecar handler: /v1/agent/ping (POST) — Python ← Rust only."""

from __future__ import annotations

import logging
from typing import Any

from shared.logging import bind_log_context

logger = logging.getLogger("opendesk.sidecar.agent")


def handle_agent_ping(payload: dict[str, Any] | None, *, trace_id: str) -> dict[str, Any]:
    """Contract: contracts/schema/v1/agent/sidecar/ping.*.schema.json"""
    with bind_log_context(trace_id=trace_id, feature="agent"):
        logger.info("handle_agent_ping", extra={"event": "agent.ping"})
        _ = payload
    return {"ok": True, "trace_id": trace_id}

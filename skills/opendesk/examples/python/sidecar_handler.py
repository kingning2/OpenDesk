"""Example: Python sidecar handler (Python ← Rust only).

Real handlers live in python/packages/gateway/src/gateway/handlers/.
"""

from __future__ import annotations

import logging
from typing import Any

logger = logging.getLogger("opendesk.examples.sidecar")


def handle_agent_ping(payload: dict[str, Any] | None, *, trace_id: str) -> dict[str, Any]:
    """POST /v1/agent/ping — contract-driven request/response."""
    logger.info("agent_ping", extra={"trace_id": trace_id})
    _ = payload
    return {"ok": True, "trace_id": trace_id}

"""Sidecar handler: /v1/agent/ping (POST) — Python ← Rust only."""

from __future__ import annotations

from typing import Any

from shared.logging import bind_log_context


def handle_agent_ping(payload: dict[str, Any] | None, *, trace_id: str) -> dict[str, Any]:
    """Contract: contracts/schema/v1/agent/sidecar/ping.*.schema.json"""
    with bind_log_context(trace_id=trace_id, feature="agent"):
        # 健康检查高频调用，不打日志，避免刷屏。
        _ = payload
    return {"ok": True, "trace_id": trace_id}

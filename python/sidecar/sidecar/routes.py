"""Sidecar route table — consumed by Rust-managed HTTP server."""

from __future__ import annotations

ROUTES: dict[str, tuple[str, str]] = {}

ROUTES["/v1/agent/ping"] = ("POST", "handle_agent_ping")

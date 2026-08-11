"""Sidecar route table — consumed by Rust-managed HTTP server."""

from __future__ import annotations

ROUTES: dict[str, tuple[str, str]] = {}

ROUTES["/v1/agent/ping"] = ("POST", "handle_agent_ping")
ROUTES["/v1/channel/login"] = ("POST", "handle_channel_login")
ROUTES["/v1/channel/qr_start"] = ("POST", "handle_qr_start")
ROUTES["/v1/channel/qr_check"] = ("POST", "handle_qr_check")
ROUTES["/v1/channel/qr_cancel"] = ("POST", "handle_qr_cancel")
ROUTES["/v1/llm/chat"] = ("POST", "handle_llm_chat")
ROUTES["/v1/llm/classify"] = ("POST", "handle_llm_classify")

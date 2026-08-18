"""Sidecar route table — consumed by Rust-managed HTTP server."""

from __future__ import annotations

ROUTES: dict[str, tuple[str, str]] = {}

ROUTES["/v1/channel/password_login"] = ("POST", "handle_password_login")
ROUTES["/v1/channel/qr_start"] = ("POST", "handle_qr_start")
ROUTES["/v1/channel/qr_check"] = ("POST", "handle_qr_check")
ROUTES["/v1/channel/qr_cancel"] = ("POST", "handle_qr_cancel")

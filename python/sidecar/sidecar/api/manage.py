"""Sidecar 管理面 GET 路由（health / stats / metrics 等）。

作者：Xiaoman
创建时间：2026-07-22
"""

from __future__ import annotations

from typing import Any

from sidecar.api.http_io import send_json, send_text
from sidecar.api.routes import ROUTES


def handle_manage_get(handler: Any, path: str) -> bool:
    """处理管理面 GET；命中则写响应并返回 ``True``。

    作者：Xiaoman
    创建时间：2026-07-22

    Args:
        handler: ``BaseHTTPRequestHandler`` 实例。
        path: 请求路径。

    Returns:
        是否已处理该路径。
    """
    if path == "/health":
        send_json(handler, 200, {"status": "ok"})
        return True
    if path == "/stats":
        send_json(handler, 200, {"uptime_ms": 0, "requests": 0})
        return True
    if path == "/tasks/active":
        send_json(handler, 200, {"tasks": []})
        return True
    if path == "/debug/dump":
        send_json(handler, 200, {"routes": list(ROUTES.keys())})
        return True
    if path == "/metrics":
        send_text(handler, 200, "# opendesk sidecar metrics (skeleton)\n")
        return True
    return False

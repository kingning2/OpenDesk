"""Sidecar HTTP 请求处理（GET 管理面 + POST 业务路由）。

作者：Xiaoman
创建时间：2026-07-22
"""

from __future__ import annotations

import logging
from http.server import BaseHTTPRequestHandler
from typing import ClassVar

from sidecar.api.http_io import read_json_body, send_json
from sidecar.api.manage import handle_manage_get
from sidecar.api.registry import HANDLERS
from sidecar.api.routes import ROUTES

logger = logging.getLogger("opendesk.sidecar")


class SidecarHandler(BaseHTTPRequestHandler):
    """Rust 生命周期托管的 Sidecar HTTP Handler。

    作者：Xiaoman
    创建时间：2026-07-22
    """

    routes: ClassVar[dict[str, tuple[str, str]]] = ROUTES

    def log_message(self, format: str, *args: object) -> None:
        """将访问日志转到 sidecar logger。

        作者：Xiaoman
        创建时间：2026-07-22
        """
        logger.info("%s - %s", self.address_string(), format % args)

    def do_GET(self) -> None:
        """处理管理面 GET。

        作者：Xiaoman
        创建时间：2026-07-22
        """
        if handle_manage_get(self, self.path):
            return
        send_json(self, 404, {"code": "not_found", "message": "route not found"})

    def do_POST(self) -> None:
        """按路由表分发业务 POST。

        作者：Xiaoman
        创建时间：2026-07-22
        """
        route = ROUTES.get(self.path)
        if route is None:
            send_json(self, 404, {"code": "not_found", "message": "route not found"})
            return
        method, handler_name = route
        if method != "POST":
            send_json(
                self,
                405,
                {"code": "method_not_allowed", "message": "method not allowed"},
            )
            return
        handler = HANDLERS.get(handler_name)
        if handler is None:
            send_json(
                self,
                500,
                {"code": "handler_missing", "message": "handler not registered"},
            )
            return
        payload = read_json_body(self)
        trace_id = ""
        if isinstance(payload, dict):
            trace_id = str(payload.get("trace_id", ""))
        result = handler(payload if isinstance(payload, dict) else None, trace_id=trace_id)
        send_json(self, 200, result)

"""Sidecar HTTP server — consumed by Rust-managed lifecycle."""

from __future__ import annotations

import json
import logging
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Any, ClassVar

from gateway.handlers import handle_agent_ping
from sidecar.routes import ROUTES

logger = logging.getLogger("opendesk.sidecar")

HANDLERS = {
    "handle_agent_ping": handle_agent_ping,
}


class SidecarHandler(BaseHTTPRequestHandler):
    routes: ClassVar[dict[str, tuple[str, str]]] = ROUTES

    def log_message(self, format: str, *args: object) -> None:
        logger.info("%s - %s", self.address_string(), format % args)

    def do_GET(self) -> None:
        if self.path == "/health":
            self._send_json(200, {"status": "ok"})
            return
        if self.path == "/stats":
            self._send_json(200, {"uptime_ms": 0, "requests": 0})
            return
        if self.path == "/tasks/active":
            self._send_json(200, {"tasks": []})
            return
        if self.path == "/debug/dump":
            self._send_json(200, {"routes": list(ROUTES.keys())})
            return
        if self.path == "/metrics":
            self._send_text(200, "# opendesk sidecar metrics (skeleton)\n")
            return
        self._send_json(404, {"code": "not_found", "message": "route not found"})

    def do_POST(self) -> None:
        route = ROUTES.get(self.path)
        if route is None:
            self._send_json(404, {"code": "not_found", "message": "route not found"})
            return
        method, handler_name = route
        if method != "POST":
            self._send_json(405, {"code": "method_not_allowed", "message": "method not allowed"})
            return
        handler = HANDLERS.get(handler_name)
        if handler is None:
            self._send_json(500, {"code": "handler_missing", "message": "handler not registered"})
            return
        payload = self._read_json()
        trace_id = ""
        if isinstance(payload, dict):
            trace_id = str(payload.get("trace_id", ""))
        result = handler(payload if isinstance(payload, dict) else None, trace_id=trace_id)
        self._send_json(200, result)

    def _read_json(self) -> Any:
        length = int(self.headers.get("Content-Length", "0"))
        if length == 0:
            return None
        raw = self.rfile.read(length)
        return json.loads(raw.decode("utf-8"))

    def _send_json(self, status: int, payload: dict[str, Any]) -> None:
        body = json.dumps(payload).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _send_text(self, status: int, payload: str) -> None:
        body = payload.encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "text/plain")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def serve(port: int = 8787) -> None:
    server = ThreadingHTTPServer(("127.0.0.1", port), SidecarHandler)
    logger.info("sidecar listening on 127.0.0.1:%s", port)
    server.serve_forever()

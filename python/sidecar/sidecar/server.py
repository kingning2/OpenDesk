"""Sidecar HTTP server — consumed by Rust-managed lifecycle."""

from __future__ import annotations

import asyncio
import inspect
import json
import logging
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Any, ClassVar

from gateway.handlers import (
    handle_agent_ping,
    handle_llm_chat,
    handle_llm_classify,
    handle_qr_cancel,
    handle_qr_check,
    handle_qr_start,
)
from sidecar.routes import ROUTES

logger = logging.getLogger("opendesk.sidecar")

HANDLERS = {
    "handle_agent_ping": handle_agent_ping,
    "handle_qr_start": handle_qr_start,
    "handle_qr_check": handle_qr_check,
    "handle_qr_cancel": handle_qr_cancel,
    "handle_llm_chat": handle_llm_chat,
    "handle_llm_classify": handle_llm_classify,
}

# 常驻后台事件循环（单例）：所有异步 handler 共用同一个 loop。
# 之前每个请求新建/关闭 loop，导致 Playwright 页面跨请求复用失败
# （'NoneType' object has no attribute 'send'），扫码检测静默失效。
_ASYNC_LOOP: asyncio.AbstractEventLoop | None = None
_ASYNC_LOOP_LOCK = threading.Lock()


def _get_async_loop() -> asyncio.AbstractEventLoop:
    """获取（惰性创建）常驻后台事件循环。"""
    global _ASYNC_LOOP
    if _ASYNC_LOOP is None or _ASYNC_LOOP.is_closed():
        with _ASYNC_LOOP_LOCK:
            if _ASYNC_LOOP is None or _ASYNC_LOOP.is_closed():
                loop = asyncio.new_event_loop()

                def _run() -> None:
                    asyncio.set_event_loop(loop)
                    loop.run_forever()

                threading.Thread(target=_run, daemon=True, name="sidecar-asyncio").start()
                _ASYNC_LOOP = loop
    return _ASYNC_LOOP


class SidecarHandler(BaseHTTPRequestHandler):
    routes: ClassVar[dict[str, tuple[str, str]]] = ROUTES

    def log_message(self, format: str, *args: object) -> None:
        # 不打印每个请求：扫码轮询每 2s 一次，会刷屏。仅靠 handler 层关键事件日志。
        del format, args

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
        # 异步 handler（如 Playwright 登录）跑在常驻事件循环上，
        # 保证跨请求复用的页面/浏览器不因旧 loop 关闭而失效。
        if inspect.iscoroutine(result):
            loop = _get_async_loop()
            result = asyncio.run_coroutine_threadsafe(result, loop).result()
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
    try:
        server.serve_forever()
    except Exception:
        logger.exception(
            "侧车服务异常",
            extra={"event": "sidecar.failed", "feature": "runtime"},
        )
        raise
    finally:
        server.server_close()

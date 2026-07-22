"""Sidecar HTTP 读写辅助（JSON / 纯文本响应）。

作者：Xiaoman
创建时间：2026-07-22
"""

from __future__ import annotations

import json
from typing import Any


def read_json_body(handler: Any) -> Any:
    """从请求体读取 JSON。

    作者：Xiaoman
    创建时间：2026-07-22

    Args:
        handler: ``BaseHTTPRequestHandler`` 实例。

    Returns:
        解析后的对象；无 body 时返回 ``None``。
    """
    length = int(handler.headers.get("Content-Length", "0"))
    if length == 0:
        return None
    raw = handler.rfile.read(length)
    return json.loads(raw.decode("utf-8"))


def send_json(handler: Any, status: int, payload: dict[str, Any]) -> None:
    """写入 JSON 响应。

    作者：Xiaoman
    创建时间：2026-07-22

    Args:
        handler: ``BaseHTTPRequestHandler`` 实例。
        status: HTTP 状态码。
        payload: 响应字典。
    """
    body = json.dumps(payload).encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json")
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


def send_text(handler: Any, status: int, payload: str) -> None:
    """写入纯文本响应。

    作者：Xiaoman
    创建时间：2026-07-22

    Args:
        handler: ``BaseHTTPRequestHandler`` 实例。
        status: HTTP 状态码。
        payload: 文本内容。
    """
    body = payload.encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", "text/plain")
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)

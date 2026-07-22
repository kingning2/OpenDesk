"""Sidecar HTTP 服务启动（由 Rust lifecycle 拉起）。

作者：Xiaoman
创建时间：2026-07-22
"""

from __future__ import annotations

import logging
from http.server import ThreadingHTTPServer

from sidecar.api.handler import SidecarHandler

logger = logging.getLogger("opendesk.sidecar")


def serve(port: int = 8787) -> None:
    """在本地端口启动 Sidecar HTTP 服务并阻塞。

    作者：Xiaoman
    创建时间：2026-07-22

    Args:
        port: 监听端口，默认 ``8787``。
    """
    server = ThreadingHTTPServer(("127.0.0.1", port), SidecarHandler)
    logger.info(
        "sidecar ready",
        extra={"event": "sidecar.ready", "feature": "runtime", "port": port},
    )
    try:
        server.serve_forever()
    except Exception:
        logger.exception(
            "sidecar server failed",
            extra={"event": "sidecar.failed", "feature": "runtime"},
        )
        raise
    finally:
        server.server_close()
        logger.info(
            "sidecar stopped",
            extra={"event": "sidecar.stopped", "feature": "runtime"},
        )

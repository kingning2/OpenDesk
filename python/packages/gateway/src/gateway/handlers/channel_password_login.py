"""Sidecar handler: /v1/channel/password_login (POST) — Python ← Rust only。"""

from __future__ import annotations

import logging
import time
from typing import Any

from gateway.login.password_login import password_login_by_platform
from shared.logging import bind_log_context

logger = logging.getLogger("dingda.sidecar.password-login")


async def handle_password_login(payload: dict[str, Any] | None, *, trace_id: str) -> dict[str, Any]:
    """使用账号密码登录闲鱼并导出 cookies。"""
    with bind_log_context(trace_id=trace_id, feature="channel"):
        platform = str((payload or {}).get("platform") or "xianyu").strip().lower() or "xianyu"
        login_id = str((payload or {}).get("login_id") or "").strip()
        password = str((payload or {}).get("password") or "")
        if not login_id or not password:
            return {
                "ok": False,
                "status": "error",
                "cookies": None,
                "detail": "缺少 login_id 或 password",
                "trace_id": trace_id,
            }

        started = time.perf_counter()
        ok, detail, data = await password_login_by_platform(login_id, password, platform=platform)
        duration_ms = max(0, int((time.perf_counter() - started) * 1000))
        logger.info(
            "账号密码登录完成 status=%s duration_ms=%s",
            data.get("status", "error"),
            duration_ms,
            extra={
                "event": "channel.password_login.completed",
                "status": data.get("status", "error"),
                "duration_ms": duration_ms,
                "platform": platform,
            },
        )
        return {
            "ok": ok,
            "status": data.get("status", "error"),
            "cookies": data.get("cookies"),
            "detail": detail,
            "trace_id": trace_id,
        }

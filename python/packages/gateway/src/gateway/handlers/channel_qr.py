"""Sidecar handlers: /v1/channel/qr_start / qr_check / qr_cancel (POST) — Python ← Rust only。

闲鱼扫码登录：Playwright 打开登录页显示二维码，前端展示后轮询扫码状态，
登录成功后导出 cookies 返回 Rust。
"""

from __future__ import annotations

import logging
from typing import Any

from gateway.login.qr_session import (
    cancel_qr_login,
    check_qr_login,
    start_qr_login,
)
from shared.logging import bind_log_context

logger = logging.getLogger("opendesk.sidecar.qr")


async def handle_qr_start(
    payload: dict[str, Any] | None, *, trace_id: str
) -> dict[str, Any]:
    """Contract: contracts/schema/v1/channel/sidecar/qr_start.*.schema.json"""
    with bind_log_context(trace_id=trace_id, feature="channel"):
        ok, detail, data = await start_qr_login()
        logger.info("channel qr start", extra={"event": "channel.qr.start", "ok": ok})
        return {
            "ok": ok,
            "status": data.get("status", "error"),
            "session_id": data.get("session_id"),
            "qr_base64": data.get("qr_base64"),
            "detail": detail,
            "trace_id": trace_id,
        }


async def handle_qr_check(
    payload: dict[str, Any] | None, *, trace_id: str
) -> dict[str, Any]:
    """Contract: contracts/schema/v1/channel/sidecar/qr_check.*.schema.json"""
    with bind_log_context(trace_id=trace_id, feature="channel"):
        session_id = (payload or {}).get("session_id") or ""
        if not session_id:
            return {
                "ok": False,
                "status": "error",
                "session_id": None,
                "cookies": None,
                "detail": "缺少 session_id",
                "trace_id": trace_id,
            }
        ok, detail, data = await check_qr_login(session_id)
        logger.info(
            "channel qr check",
            extra={"event": "channel.qr.check", "ok": ok, "status": data.get("status")},
        )
        return {
            "ok": ok,
            "status": data.get("status", "error"),
            "session_id": session_id,
            "cookies": data.get("cookies"),
            "detail": detail,
            "trace_id": trace_id,
        }


async def handle_qr_cancel(
    payload: dict[str, Any] | None, *, trace_id: str
) -> dict[str, Any]:
    """Contract: contracts/schema/v1/channel/sidecar/qr_cancel.*.schema.json"""
    with bind_log_context(trace_id=trace_id, feature="channel"):
        session_id = (payload or {}).get("session_id") or ""
        ok, detail, _data = await cancel_qr_login(session_id)
        return {
            "ok": ok,
            "detail": detail,
            "trace_id": trace_id,
        }

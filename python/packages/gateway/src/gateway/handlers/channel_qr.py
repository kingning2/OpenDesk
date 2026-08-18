"""Sidecar handlers: /v1/channel/qr_start / qr_check / qr_cancel (POST) — Python ← Rust only。

闲鱼扫码登录：Playwright 打开登录页显示二维码，前端展示后轮询扫码状态，
登录成功后导出 cookies 返回 Rust。
"""

from __future__ import annotations

import logging
import time
from typing import Any

from gateway.login.qr_session import (
    cancel_qr_login_by_platform,
    check_qr_login_by_platform,
    start_qr_login_by_platform,
)
from shared.logging import bind_log_context

logger = logging.getLogger("opendesk.sidecar.qr")


async def handle_qr_start(payload: dict[str, Any] | None, *, trace_id: str) -> dict[str, Any]:
    """Contract: contracts/schema/v1/channel/sidecar/qr_start.*.schema.json"""
    with bind_log_context(trace_id=trace_id, feature="channel"):
        platform = str((payload or {}).get("platform") or "xianyu").strip().lower() or "xianyu"
        started = time.perf_counter()
        ok, detail, data = await start_qr_login_by_platform(platform)
        duration_ms = max(0, int((time.perf_counter() - started) * 1000))
        qr = data.get("qr_base64")
        status = data.get("status", "error")
        if not qr:
            # 启动失败：start_qr_login 内部已打关键步骤日志，这里补一条失败结果。
            logger.warning(
                "获取二维码失败：%s duration_ms=%s",
                detail,
                duration_ms,
                extra={
                    "event": "channel.qr.failed",
                    "status": status,
                    "duration_ms": duration_ms,
                    "platform": platform,
                },
            )
        else:
            logger.info(
                "获取二维码成功 status=%s duration_ms=%s",
                status,
                duration_ms,
                extra={
                    "event": "channel.qr.started",
                    "status": status,
                    "duration_ms": duration_ms,
                    "session_id": data.get("session_id"),
                    "platform": platform,
                },
            )
        return {
            "ok": ok,
            "status": status,
            "session_id": data.get("session_id"),
            "qr_base64": qr,
            "detail": detail,
            "trace_id": trace_id,
        }


async def handle_qr_check(payload: dict[str, Any] | None, *, trace_id: str) -> dict[str, Any]:
    """Contract: contracts/schema/v1/channel/sidecar/qr_check.*.schema.json"""
    with bind_log_context(trace_id=trace_id, feature="channel"):
        platform = str((payload or {}).get("platform") or "xianyu").strip().lower() or "xianyu"
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
        started = time.perf_counter()
        ok, detail, data = await check_qr_login_by_platform(session_id, platform=platform)
        duration_ms = max(0, int((time.perf_counter() - started) * 1000))
        status = data.get("status")
        # 等待扫码是常态，不逐次打扰；只在状态变化（扫码/确认/成功/刷新/过期/失败）时打一条。
        if status != "waiting":
            logger.info(
                "%s duration_ms=%s",
                detail,
                duration_ms,
                extra={
                    "event": "channel.qr.check",
                    "status": status,
                    "duration_ms": duration_ms,
                    "session_id": session_id,
                    "platform": platform,
                },
            )
        return {
            "ok": ok,
            "status": data.get("status", "error"),
            "session_id": session_id,
            "cookies": data.get("cookies"),
            "detail": detail,
            "qr_base64": data.get("qr_base64"),
            "trace_id": trace_id,
        }


async def handle_qr_cancel(payload: dict[str, Any] | None, *, trace_id: str) -> dict[str, Any]:
    """Contract: contracts/schema/v1/channel/sidecar/qr_cancel.*.schema.json"""
    with bind_log_context(trace_id=trace_id, feature="channel"):
        platform = str((payload or {}).get("platform") or "xianyu").strip().lower() or "xianyu"
        session_id = (payload or {}).get("session_id") or ""
        started = time.perf_counter()
        ok, detail, _data = await cancel_qr_login_by_platform(session_id, platform=platform)
        duration_ms = max(0, int((time.perf_counter() - started) * 1000))
        logger.info(
            "取消扫码登录 ok=%s duration_ms=%s",
            ok,
            duration_ms,
            extra={
                "event": "channel.qr.cancelled",
                "ok": ok,
                "duration_ms": duration_ms,
                "session_id": session_id or None,
                "platform": platform,
            },
        )
        return {
            "ok": ok,
            "detail": detail,
            "trace_id": trace_id,
        }

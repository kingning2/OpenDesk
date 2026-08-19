"""Sidecar handler: /v1/channel/cookie_renew (POST) — Python ← Rust only。

闲鱼风控后续期：注入现有 Cookie，临时打开 Playwright，导出新 Cookie。

作者：Xiaoman
创建时间：2026-08-19
"""

from __future__ import annotations

import logging
import time
from typing import Any

from gateway.login.cookie_renew import renew_cookies
from shared.logging import bind_log_context

logger = logging.getLogger("dingda.sidecar.cookie-renew")


async def handle_cookie_renew(payload: dict[str, Any] | None, *, trace_id: str) -> dict[str, Any]:
    """Contract: contracts/schema/v1/channel/sidecar/cookie_renew.*.schema.json"""
    with bind_log_context(trace_id=trace_id, feature="channel"):
        body = payload or {}
        account_id = str(body.get("account_id") or "").strip()
        cookies = body.get("cookies")
        if not account_id or not isinstance(cookies, list):
            return {
                "ok": False,
                "status": "error",
                "cookies": None,
                "detail": "缺少 account_id 或 cookies",
                "trace_id": trace_id,
            }

        punish_url = str(body.get("punish_url") or "").strip() or None
        started = time.perf_counter()
        ok, detail, data = await renew_cookies(
            cookies,
            account_id=account_id,
            punish_url=punish_url,
        )
        duration_ms = max(0, int((time.perf_counter() - started) * 1000))
        logger.info(
            "Cookie 浏览器续期完成 status=%s duration_ms=%s account=%s",
            data.get("status", "error"),
            duration_ms,
            account_id,
            extra={
                "event": "channel.cookie_renew.completed",
                "status": data.get("status", "error"),
                "duration_ms": duration_ms,
                "account_id": account_id,
            },
        )
        return {
            "ok": ok,
            "status": data.get("status", "error"),
            "cookies": data.get("cookies"),
            "detail": detail,
            "trace_id": trace_id,
        }

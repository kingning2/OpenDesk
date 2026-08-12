"""Sidecar handler: /v1/channel/login (POST) — Python ← Rust only。

用 Playwright 从浏览器快照恢复闲鱼会话，打开 goofish.com 校验登录态，
导出登录后的 cookies 返回给 Rust（Rust 再用这些 cookies 走 WS 协议收发）。
"""

from __future__ import annotations

import logging
from typing import Any

from gateway.login.snapshot import (
    ANTI_DETECT_SCRIPT,
    build_context_overrides,
    build_extra_headers,
    build_storage_state,
    parse_snapshot,
)
from shared.logging import bind_log_context

logger = logging.getLogger("opendesk.sidecar.login")

try:  # playwright 为可选运行时依赖；缺失或损坏时返回可读错误。
    from playwright.async_api import async_playwright
except Exception:  # pragma: no cover — ImportError / 损坏扩展（如 greenlet）
    async_playwright = None  # type: ignore[assignment]

LOGIN_CHECK_URL = "https://www.goofish.com/"


def _to_serializable_cookies(cookies: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """把 Playwright cookie 对象转换为可序列化结构（保留契约字段）。"""
    return [
        {
            "name": c.get("name", ""),
            "value": c.get("value", ""),
            "domain": c.get("domain", ""),
            "path": c.get("path", ""),
            "expires": c.get("expires"),
            "httpOnly": c.get("httpOnly", False),
            "secure": c.get("secure", False),
            "sameSite": c.get("sameSite") or "Lax",
        }
        for c in cookies
        if c.get("name") and c.get("value")
    ]


async def _perform_login(credential: str) -> tuple[bool, str, list[dict[str, Any]]]:
    """用快照恢复会话并导出登录后 cookies。返回 (ok, detail, cookies)。"""
    if async_playwright is None:
        msg = "playwright 未安装：请运行 uv add playwright 并执行 playwright install chromium"
        return False, msg, []

    snapshot = parse_snapshot(credential)
    cookies = snapshot.get("cookies") or []
    if not cookies:
        return False, "凭据中缺少 cookies（请先通过 Chrome 扩展导出快照）", []

    launch_args = [
        "--disable-blink-features=AutomationControlled",
        "--disable-dev-shm-usage",
        "--no-sandbox",
        "--disable-setuid-sandbox",
    ]

    async with async_playwright() as playwright:
        browser = await playwright.chromium.launch(
            headless=True,
            args=launch_args,
        )

        context_kwargs: dict[str, Any] = {"storage_state": build_storage_state(snapshot)}
        context_kwargs.update(build_context_overrides(snapshot))
        extra_headers = build_extra_headers(snapshot.get("headers"))
        if extra_headers:
            context_kwargs["extra_http_headers"] = extra_headers

        context = await browser.new_context(**context_kwargs)
        await context.add_init_script(ANTI_DETECT_SCRIPT)

        page = await context.new_page()
        try:
            await page.goto(LOGIN_CHECK_URL, wait_until="domcontentloaded", timeout=30000)
        except Exception as error:  # noqa: BLE001
            await browser.close()
            return False, f"打开闲鱼页面失败: {error}", []

        # 登录态校验：重定向到 login 页或 URL 含 login 标识则判定失效。
        current_url = page.url
        logged_in = (
            "login" not in current_url.lower()
            and "passport" not in current_url.lower()
            and "goofish.com" in current_url.lower()
        )

        if not logged_in:
            await browser.close()
            return False, f"登录态失效，被重定向到 {current_url}", []

        exported = _to_serializable_cookies(await context.cookies())
        await browser.close()
        if not exported:
            return False, "导出 cookies 为空，登录态可能已失效", []
        return True, "登录成功，已导出 cookies", exported


async def handle_channel_login(payload: dict[str, Any] | None, *, trace_id: str) -> dict[str, Any]:
    """Contract: contracts/schema/v1/channel/sidecar/login.*.schema.json"""
    with bind_log_context(trace_id=trace_id, feature="channel"):
        if payload is None:
            return {
                "ok": False,
                "state": "error",
                "cookies": None,
                "detail": "缺少参数",
                "trace_id": trace_id,
            }
        account_id = payload.get("account_id") or ""
        credential = payload.get("credential") or ""

        if not account_id or not credential:
            return {
                "ok": False,
                "state": "error",
                "cookies": None,
                "detail": "缺少 account_id 或 credential",
                "trace_id": trace_id,
            }

        ok, detail, cookies = await _perform_login(credential)
        logger.info(
            "channel login",
            extra={"event": "channel.login", "account_id": account_id, "ok": ok},
        )
        return {
            "ok": ok,
            "state": "connected" if ok else "error",
            "cookies": cookies if ok else None,
            "detail": detail,
            "trace_id": trace_id,
        }

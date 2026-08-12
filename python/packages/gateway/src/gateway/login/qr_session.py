"""闲鱼扫码登录会话管理 — Playwright 打开登录页，截图二维码，轮询扫码状态。

闲鱼/淘宝网页版登录为阿里统一二维码登录：
- 打开 goofish.com → 未登录跳转淘宝登录页（显示二维码）
- 用户用 App 扫码 → 浏览器写入 cookies → 跳回原页
本模块跨 HTTP 请求保存 Playwright 会话，供 start/check/cancel 使用。
"""

from __future__ import annotations

import base64
import contextlib
import logging
import time
import uuid
from typing import Any

from gateway.login.snapshot import ANTI_DETECT_SCRIPT

logger = logging.getLogger("opendesk.sidecar.qr")

try:  # playwright 为可选运行时依赖；缺失或损坏时降级。
    from playwright.async_api import Browser, BrowserContext, Page, async_playwright
except Exception:  # pragma: no cover — ImportError / 损坏扩展（如 greenlet）
    async_playwright = None  # type: ignore[assignment]
    Browser = Any  # type: ignore[assignment,misc]
    BrowserContext = Any  # type: ignore[assignment,misc]
    Page = Any  # type: ignore[assignment,misc]

# 登录页 / 目标页。
LOGIN_ENTRY_URL = "https://www.goofish.com/"
# 二维码元素选择器（淘宝/闲鱼登录页通用，兜底用图片扫描）。
QR_SELECTORS = [
    "img.qrcode-img",
    "img[class*='qrcode']",
    "img[id*='qrCode']",
    ".icon-qrcode img",
    "#login img",
]

LAUNCH_ARGS = [
    "--disable-blink-features=AutomationControlled",
    "--disable-dev-shm-usage",
    "--no-sandbox",
    "--disable-setuid-sandbox",
]

# 扫码状态机。
STATUS_GENERATING = "generating"
STATUS_READY = "ready"
STATUS_WAITING = "waiting"
STATUS_SCANNED = "scanned"
STATUS_CONFIRMED = "confirmed"
STATUS_SUCCESS = "success"
STATUS_EXPIRED = "expired"
STATUS_FAILED = "failed"

# 二维码过期时间（秒）。
QR_EXPIRE_SECONDS = 300


class QrSession:
    """一次扫码登录的 Playwright 会话。"""

    def __init__(self, session_id: str) -> None:
        self.session_id = session_id
        self.browser: Browser | None = None
        self.context: BrowserContext | None = None
        self.page: Page | None = None
        self.status = STATUS_GENERATING
        self.started_at = time.monotonic()
        self.detail = ""

    def expired(self) -> bool:
        return time.monotonic() - self.started_at > QR_EXPIRE_SECONDS

    async def close(self) -> None:
        with contextlib.suppress(Exception):
            if self.context:
                await self.context.close()
        with contextlib.suppress(Exception):
            if self.browser:
                await self.browser.close()
        self.browser = None
        self.context = None
        self.page = None


# 全局进行中的扫码会话（模块级，跨 HTTP 请求共享）。
QR_SESSIONS: dict[str, QrSession] = {}


def _to_serializable_cookies(cookies: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """把 Playwright cookie 对象转换为可序列化结构。"""
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


def _to_base64(screenshot: bytes) -> str:
    """截图字节 → data URL。"""
    return "data:image/png;base64," + base64.b64encode(screenshot).decode("ascii")


async def start_qr_login() -> tuple[bool, str, dict[str, Any]]:
    """启动扫码登录：打开登录页，截图二维码。返回 (ok, detail, payload)。"""
    if async_playwright is None:
        msg = "playwright 未安装：请运行 uv add playwright 并执行 playwright install chromium"
        return False, msg, {"status": STATUS_FAILED}

    session_id = str(uuid.uuid4())
    session = QrSession(session_id)

    try:
        playwright = await async_playwright().start()
        browser = await playwright.chromium.launch(headless=True, args=LAUNCH_ARGS)
        context = await browser.new_context()
        await context.add_init_script(ANTI_DETECT_SCRIPT)
        page = await context.new_page()

        session.browser = browser
        session.context = context
        session.page = page
        session.status = STATUS_GENERATING

        await page.goto(LOGIN_ENTRY_URL, wait_until="domcontentloaded", timeout=30000)

        # 等二维码元素出现。
        qr_element = None
        for selector in QR_SELECTORS:
            locator = page.locator(selector).first
            try:
                await locator.wait_for(state="visible", timeout=8000)
                qr_element = locator
                break
            except Exception:  # noqa: BLE001
                continue

        if qr_element is None:
            # 兜底：截图整个页面区域（可能二维码不是 img 元素）。
            with contextlib.suppress(Exception):
                await page.wait_for_timeout(2000)
                screenshot = await page.screenshot()
                if screenshot:
                    session.status = STATUS_READY
                    session.detail = "已显示登录页（兜底截图）"
                    QR_SESSIONS[session_id] = session
                    return (
                        True,
                        "已显示登录页",
                        {
                            "status": STATUS_READY,
                            "session_id": session_id,
                            "qr_base64": _to_base64(screenshot),
                        },
                    )

        screenshot = await qr_element.screenshot()
        session.status = STATUS_READY
        session.detail = "二维码已就绪"
        QR_SESSIONS[session_id] = session
        return (
            True,
            "二维码已就绪",
            {
                "status": STATUS_READY,
                "session_id": session_id,
                "qr_base64": _to_base64(screenshot),
            },
        )

    except Exception as error:  # noqa: BLE001
        await session.close()
        logger.exception("qr start failed", extra={"event": "channel.qr.start_failed"})
        return False, f"启动扫码登录失败: {error}", {"status": STATUS_FAILED}


async def check_qr_login(session_id: str) -> tuple[bool, str, dict[str, Any]]:
    """轮询扫码状态。登录成功后导出 cookies 并清理会话。"""
    session = QR_SESSIONS.get(session_id)
    if session is None:
        return False, "会话不存在或已过期", {"status": STATUS_FAILED}

    if session.expired():
        await session.close()
        QR_SESSIONS.pop(session_id, None)
        return True, "二维码已过期", {"status": STATUS_EXPIRED}

    try:
        page = session.page
        if page is None:
            return True, "等待中", {"status": STATUS_WAITING}

        url = page.url.lower()

        # 登录成功：URL 回到 goofish.com 且不含登录跳转。
        logged_in = "goofish.com" in url and "login" not in url and "passport" not in url

        if logged_in:
            cookies = await session.context.cookies()
            exported = _to_serializable_cookies(cookies)
            await session.close()
            QR_SESSIONS.pop(session_id, None)
            if not exported:
                return False, "登录后导出 cookies 为空", {"status": STATUS_FAILED}
            return (
                True,
                "登录成功，已提取 cookies",
                {
                    "status": STATUS_SUCCESS,
                    "cookies": exported,
                },
            )

        # 仍在登录页：尝试读取扫码状态文案。
        page_text = ""
        with contextlib.suppress(Exception):
            page_text = (await page.locator("body").inner_text(timeout=2000))[:300]

        if "确认" in page_text and ("已扫描" in page_text or "扫描成功" in page_text):
            return True, "已扫码，请在手机确认", {"status": STATUS_SCANNED}
        if "确认登录" in page_text or "确认" in page_text and "二维码" in page_text:
            return True, "已扫码，请在手机确认", {"status": STATUS_CONFIRMED}

        return True, "等待扫码", {"status": STATUS_WAITING}

    except Exception as error:  # noqa: BLE001
        await session.close()
        QR_SESSIONS.pop(session_id, None)
        logger.exception("qr check failed", extra={"event": "channel.qr.check_failed"})
        return False, f"扫码状态检查失败: {error}", {"status": STATUS_FAILED}


async def cancel_qr_login(session_id: str) -> tuple[bool, str, dict[str, Any]]:
    """取消扫码登录：关闭浏览器，清理会话。"""
    session = QR_SESSIONS.pop(session_id, None)
    if session is None:
        return True, "会话不存在", {"ok": True}
    await session.close()
    return True, "已取消", {"ok": True}

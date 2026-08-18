"""Playwright 登录公共能力：浏览器启动、UA/代理、stealth、通用页面操作。

作者：Xiaoman
创建时间：2026-08-18
"""

from __future__ import annotations

import contextlib
import os
from typing import Any

try:  # playwright 为可选运行时依赖；缺失或损坏时降级。
    from playwright.async_api import Browser, BrowserContext, Page, async_playwright
except Exception:  # pragma: no cover — ImportError / 损坏扩展（如 greenlet）
    async_playwright = None  # type: ignore[assignment]
    Browser = Any  # type: ignore[assignment,misc]
    BrowserContext = Any  # type: ignore[assignment,misc]
    Page = Any  # type: ignore[assignment,misc]

try:  # playwright-stealth 为可选运行时依赖；缺失时沿用基础参数。
    from playwright_stealth import Stealth
except Exception:  # pragma: no cover — ImportError / 包损坏
    Stealth = None  # type: ignore[assignment]

CHROME_DESKTOP_UA = (
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
    "AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36"
)
DEFAULT_VIEWPORT = {"width": 1280, "height": 800}
LAUNCH_ARGS = [
    "--disable-blink-features=AutomationControlled",
    "--disable-dev-shm-usage",
    "--no-sandbox",
    "--disable-setuid-sandbox",
]


def to_serializable_cookies(cookies: list[dict[str, Any]]) -> list[dict[str, Any]]:
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


def resolve_user_agent(platform: str) -> str:
    """按平台解析登录 User-Agent，兼容旧闲鱼变量名。"""
    platform_key = platform.upper()
    configured = os.getenv(f"DINGDA_{platform_key}_LOGIN_USER_AGENT", "").strip()
    if not configured and platform == "xianyu":
        configured = os.getenv("DINGDA_XIANYU_LOGIN_USER_AGENT", "").strip()
    return configured or CHROME_DESKTOP_UA


def resolve_proxy(platform: str) -> dict[str, str] | None:
    """按平台解析 Playwright 代理配置，兼容旧闲鱼变量名。"""
    platform_key = platform.upper()
    server = os.getenv(f"DINGDA_{platform_key}_PROXY_SERVER", "").strip()
    username = os.getenv(f"DINGDA_{platform_key}_PROXY_USERNAME", "").strip()
    password = os.getenv(f"DINGDA_{platform_key}_PROXY_PASSWORD", "").strip()
    if platform == "xianyu" and not server:
        server = os.getenv("DINGDA_XIANYU_PROXY_SERVER", "").strip()
        username = username or os.getenv("DINGDA_XIANYU_PROXY_USERNAME", "").strip()
        password = password or os.getenv("DINGDA_XIANYU_PROXY_PASSWORD", "").strip()
    if not server:
        return None
    proxy: dict[str, str] = {"server": server}
    if username:
        proxy["username"] = username
    if password:
        proxy["password"] = password
    return proxy


async def apply_stealth(context: BrowserContext, logger: Any) -> None:
    """对 Playwright context 注入 stealth 补丁。"""
    if Stealth is None:
        logger.warning("playwright-stealth 未安装，沿用基础反检测参数")
        return
    await Stealth(init_scripts_only=True).apply_stealth_async(context)


async def try_click_first(page: Page, selectors: list[str]) -> bool:
    """依次尝试点击首个可见元素。"""
    for selector in selectors:
        try:
            locator = page.locator(selector).first
            if await locator.count() == 0:
                continue
            if await locator.is_visible():
                await locator.click(timeout=2000)
                return True
        except Exception:  # noqa: BLE001
            continue
    return False


async def fill_first(page: Page, selectors: list[str], value: str) -> bool:
    """依次尝试填充首个可见输入框。"""
    for selector in selectors:
        try:
            locator = page.locator(selector).first
            if await locator.count() == 0:
                continue
            if await locator.is_visible():
                await locator.fill(value, timeout=3000)
                return True
        except Exception:  # noqa: BLE001
            continue
    return False


async def extract_first_text(page: Page, selectors: list[str]) -> str | None:
    """读取首个可见错误文案。"""
    for selector in selectors:
        try:
            locator = page.locator(selector).first
            if await locator.count() == 0:
                continue
            if await locator.is_visible():
                text = (await locator.inner_text(timeout=1000)).strip()
                if text:
                    return text
        except Exception:  # noqa: BLE001
            continue
    return None


class BrowserSession:
    """临时浏览器会话封装，统一回收 context 与 browser。"""

    def __init__(self) -> None:
        self.browser: Browser | None = None
        self.context: BrowserContext | None = None
        self.page: Page | None = None

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

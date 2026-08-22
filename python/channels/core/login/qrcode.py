"""扫码登录抽象基类：通用流程在基类，平台差异由子类覆写 hook。"""

from __future__ import annotations

import asyncio
import contextlib
import logging
import time
import uuid
from pathlib import Path
from typing import Any

from channels.core.browser.base import BrowserPlatform
from channels.core.login.helpers import (
    click_qr_refresh,
    collect_page_text,
    cookie_domains,
    find_qr_locator,
    has_login_cookie,
    login_progress_from_text,
    qr_headless,
    screenshot_qr,
    url_looks_logged_in,
)
from channels.core.login.session import (
    QR_EXPIRE_SECONDS,
    QR_REFRESH_SECONDS,
    QR_SESSIONS,
    QR_WAIT_TIMEOUT_MS,
    STATUS_CONFIRMED,
    STATUS_EXPIRED,
    STATUS_FAILED,
    STATUS_GENERATING,
    STATUS_READY,
    STATUS_REFRESHED,
    STATUS_SCANNED,
    STATUS_SUCCESS,
    STATUS_WAITING,
    QrSession,
)
from channels.core.platform_config import PlatformConfig, get_platform_config
from channels.core.playwright_common import (
    LAUNCH_ARGS,
    apply_stealth,
    async_playwright,
    launch_persistent_chromium,
    to_serializable_cookies,
)

logger = logging.getLogger("dingda.sidecar.qr")


class QrcodeLogin:
    """扫码登录模板：启动 / 轮询 / 取消。"""

    def __init__(self, browser: BrowserPlatform) -> None:
        self._browser = browser

    @property
    def platform(self) -> str:
        return self._browser.platform_id

    @property
    def config(self) -> PlatformConfig:
        return get_platform_config(self.platform)

    async def start(self) -> tuple[bool, str, dict[str, Any]]:
        if async_playwright is None:
            msg = "playwright 未安装：请运行 uv add playwright 并执行 playwright install chromium"
            return False, msg, {"status": STATUS_FAILED}

        config = self.config
        session_id = str(uuid.uuid4())
        session = QrSession(session_id, self.platform)

        try:
            proxy = self._browser.resolve_proxy()
            headless = qr_headless()
            logger.info("启动 Playwright 引擎…")
            playwright = await async_playwright().start()
            logger.info(
                "正在打开浏览器（系统 Edge/Chrome，%s）… platform=%s proxy_enabled=%s",
                "无头" if headless else "有头",
                self.platform,
                bool(proxy),
            )
            launch_kwargs: dict[str, Any] = {
                "headless": headless,
                "args": list(LAUNCH_ARGS),
                "ignore_default_args": ["--enable-automation"],
                "viewport": {"width": 1280, "height": 800},
                "locale": "zh-CN",
                "timezone_id": "Asia/Shanghai",
            }
            if proxy:
                launch_kwargs["proxy"] = proxy
            user_data_dir = Path.cwd() / "browser_data" / f"qr_{self.platform}"
            context = await launch_persistent_chromium(playwright, user_data_dir, **launch_kwargs)
            await apply_stealth(context, logger)
            await self._browser.apply_anti_detect(context, logger)
            with contextlib.suppress(Exception):
                await context.clear_cookies()
            page = context.pages[0] if context.pages else await context.new_page()

            session.context = context
            session.page = page
            session.status = STATUS_GENERATING

            logger.info("正在加载登录页… platform=%s", self.platform)
            loaded = False
            for attempt in range(3):
                try:
                    await page.goto(
                        config.login_entry_url,
                        wait_until="domcontentloaded",
                        timeout=40000,
                    )
                    loaded = True
                    break
                except Exception as error:  # noqa: BLE001
                    logger.warning("登录页加载失败（第 %s 次）：%s", attempt + 1, str(error)[:80])
                    if attempt < 2:
                        await page.wait_for_timeout(2000)
            if not loaded:
                raise RuntimeError(f"{self.platform} 登录页加载失败")

            qr_base64: str | None = None
            deadline = time.monotonic() + QR_WAIT_TIMEOUT_MS / 1000
            while time.monotonic() < deadline:
                qr_base64 = await screenshot_qr(page, config.qr_selectors)
                loc = await find_qr_locator(page, config.qr_selectors)
                if loc is not None and qr_base64:
                    break
                await page.wait_for_timeout(400)
            if not qr_base64:
                await page.wait_for_timeout(1500)
                qr_base64 = await screenshot_qr(page, config.qr_selectors)
            if not qr_base64:
                raise RuntimeError("未检测到二维码元素")

            session.status = STATUS_READY
            session.detail = "二维码已就绪"
            session.qr_base64 = qr_base64
            QR_SESSIONS[session_id] = session
            logger.info("二维码已就绪，可扫码登录 platform=%s", self.platform)
            return (
                True,
                "二维码已就绪",
                {
                    "status": STATUS_READY,
                    "session_id": session_id,
                    "qr_base64": session.qr_base64,
                },
            )
        except Exception as error:  # noqa: BLE001
            await session.close()
            logger.exception("扫码登录启动失败", extra={"event": "channel.qr.start_failed"})
            return False, f"启动扫码登录失败: {error}", {"status": STATUS_FAILED}

    async def check(self, session_id: str) -> tuple[bool, str, dict[str, Any]]:
        session = QR_SESSIONS.get(session_id)
        if session is None:
            return False, "会话不存在或已过期", {"status": STATUS_FAILED}
        if session.platform != self.platform:
            return False, "平台与会话不匹配", {"status": STATUS_FAILED}

        async with session.lock:
            if time.monotonic() - session.started_at > QR_EXPIRE_SECONDS:
                await session.close()
                QR_SESSIONS.pop(session_id, None)
                return True, "二维码已过期", {"status": STATUS_EXPIRED}

            try:
                page = session.page
                if page is None:
                    return True, "等待中", {"status": STATUS_WAITING}

                config = self.config
                cookies = await self._read_cookies(session)
                if await self._probe_logged_in(session, page, cookies):
                    return await self._finish_success(session, session_id)

                page_text = await collect_page_text(page)
                progress = login_progress_from_text(page_text)
                if progress == "success_text":
                    return await self._finish_success(session, session_id)
                if progress == "scanned":
                    return True, "已扫码，请在手机确认", {"status": STATUS_SCANNED}
                if progress == "confirmed":
                    return True, "已扫码，请在手机确认", {"status": STATUS_CONFIRMED}

                qr_missing = await find_qr_locator(page, config.qr_selectors) is None
                due = time.monotonic() - session.last_refresh_at > QR_REFRESH_SECONDS
                if qr_missing or due:
                    new_qr = await self._refresh(session)
                    if new_qr is not None:
                        session.qr_base64 = new_qr
                        session.last_refresh_at = time.monotonic()
                        logger.info("二维码已刷新 platform=%s", self.platform)
                        return (
                            True,
                            "二维码已刷新",
                            {"status": STATUS_REFRESHED, "qr_base64": new_qr},
                        )
                    session.last_refresh_at = time.monotonic()
                    logger.warning("二维码刷新未拿到新图，继续等待 platform=%s", self.platform)

                return True, "等待扫码", {"status": STATUS_WAITING}
            except Exception as error:  # noqa: BLE001
                await session.close()
                QR_SESSIONS.pop(session_id, None)
                logger.exception("扫码状态检查失败", extra={"event": "channel.qr.check_failed"})
                return False, f"扫码状态检查失败: {error}", {"status": STATUS_FAILED}

    async def cancel(self, session_id: str) -> tuple[bool, str, dict[str, Any]]:
        session = QR_SESSIONS.pop(session_id, None)
        if session is None:
            return True, "会话不存在", {"ok": True}
        if session.platform != self.platform:
            QR_SESSIONS[session_id] = session
            return False, "平台与会话不匹配", {"ok": False}
        await session.close()
        return True, "已取消", {"ok": True}

    async def _probe_logged_in(
        self,
        session: QrSession,
        page: Any,
        cookies: list[dict[str, Any]],
    ) -> bool:
        """判定是否已登录；子类可覆写以加入平台特有逻辑（如 SSO）。"""
        config = self.config
        url = page.url.lower()
        logged_in = url_looks_logged_in(url, domain_keyword=config.cookie_domain_keyword)
        has_site = has_login_cookie(
            cookies,
            name=config.login_cookie_name,
            domain_keyword=config.cookie_domain_keyword,
        )
        return logged_in or has_site

    async def _read_cookies(self, session: QrSession) -> list[dict[str, Any]]:
        if session.context is None:
            return []
        with contextlib.suppress(Exception):
            return await session.context.cookies()
        return []

    async def _refresh(self, session: QrSession) -> str | None:
        config = self.config
        page = session.page
        if page is None:
            logger.warning("二维码刷新失败：浏览器会话已关闭")
            return None
        try:
            if await click_qr_refresh(page):
                await page.wait_for_timeout(1200)
                shot = await screenshot_qr(page, config.qr_selectors)
                if shot:
                    logger.info("二维码已点刷新 platform=%s", self.platform)
                    return shot
            await page.goto(config.login_entry_url, wait_until="domcontentloaded", timeout=40000)
            await page.wait_for_timeout(800)
            shot = await screenshot_qr(page, config.qr_selectors)
            if shot:
                return shot
            logger.warning("二维码刷新失败：截图为空")
            return None
        except Exception as error:  # noqa: BLE001
            logger.warning(
                "二维码刷新失败：%s",
                error,
                extra={"event": "channel.qr.refresh_failed", "reason": str(error)},
            )
            return None

    async def _goto(self, page: Any, url: str, *, wait_ms: int = 1500) -> str:
        logger.info("打开页面 %s", url)
        try:
            await page.goto(url, wait_until="domcontentloaded", timeout=40000)
            await page.wait_for_timeout(wait_ms)
            final = str(page.url)
            logger.info("页面落地 %s", final[:180])
            return final
        except Exception as error:  # noqa: BLE001
            logger.warning("打开页面失败 url=%s error=%s", url, str(error)[:160])
            return ""

    async def _warm_home(self, session: QrSession) -> None:
        page = session.page
        if page is None:
            return
        config = self.config
        current = str(page.url).lower()
        if (
            config.cookie_domain_keyword in current
            and "login" not in current
            and "passport" not in current
        ):
            logger.info("已在 %s 站内，跳过首页补访 url=%s", self.platform, page.url[:180])
            return
        await self._goto(page, config.home_url, wait_ms=3000)

    async def _finish_success(
        self,
        session: QrSession,
        session_id: str,
    ) -> tuple[bool, str, dict[str, Any]]:
        page = session.page
        config = self.config
        await self._warm_home(session)

        for attempt in range(3):
            if session.context is None:
                logger.warning("导出 cookies 失败：Playwright context 为空")
                break
            try:
                raw = await session.context.cookies()
                exported = to_serializable_cookies(raw)
                if exported:
                    has_login = has_login_cookie(
                        exported,
                        name=config.login_cookie_name,
                        domain_keyword=config.cookie_domain_keyword,
                    )
                    logger.info(
                        "导出 cookies platform=%s attempt=%s count=%s domains=%s has_login=%s",
                        self.platform,
                        attempt + 1,
                        len(exported),
                        cookie_domains(exported),
                        has_login,
                    )
                    if not has_login and attempt == 0 and page is not None:
                        logger.warning(
                            "导出 cookies 缺少 %s@%s，补访首页（第 %s 次）",
                            config.login_cookie_name,
                            config.cookie_domain_keyword,
                            attempt + 1,
                        )
                        await self._goto(page, config.home_url, wait_ms=3000)
                        continue
                    await session.close()
                    QR_SESSIONS.pop(session_id, None)
                    detail = (
                        f"登录成功，已提取 {self.platform} cookies"
                        if has_login
                        else f"登录成功，已提取 cookies（{self.platform} 登录 Cookie 未确认）"
                    )
                    return (
                        True,
                        detail,
                        {"status": STATUS_SUCCESS, "cookies": exported},
                    )
                logger.warning("cookies 为空，重试导出（第 %s 次）", attempt + 1)
            except Exception as error:  # noqa: BLE001
                logger.warning("导出 cookies 失败（第 %s 次）：%s", attempt + 1, str(error)[:100])
            await asyncio.sleep(1)

        return True, "已确认登录，正在提取 cookies", {"status": STATUS_CONFIRMED}

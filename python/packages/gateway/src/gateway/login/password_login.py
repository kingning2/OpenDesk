"""账号密码登录（password）——按平台配置提交表单并导出 cookies。

作者：Xiaoman
创建时间：2026-08-18
"""

from __future__ import annotations

import asyncio
import contextlib
import logging
import time
from typing import Any

from gateway.login.platform_config import get_platform_config, normalize_platform
from gateway.login.playwright_common import (
    BrowserSession,
    apply_stealth,
    async_playwright,
    extract_first_text,
    fill_first,
    resolve_proxy,
    resolve_user_agent,
    to_serializable_cookies,
    try_click_first,
)

logger = logging.getLogger("opendesk.sidecar.password-login")
STATUS_SUCCESS = "success"
STATUS_FAILED = "failed"


async def _finish_password_success(
    session: BrowserSession,
    *,
    platform: str,
) -> tuple[bool, str, dict[str, Any]]:
    """密码登录成功后阻塞等待 cookies 稳定导出。"""
    config = get_platform_config(platform)
    page = session.page
    if page is not None:
        with contextlib.suppress(Exception):
            await page.goto(
                config.home_url,
                wait_until="domcontentloaded",
                timeout=40000,
            )
            await page.wait_for_timeout(3000)

    for attempt in range(5):
        try:
            cookies = await session.context.cookies()
            exported = to_serializable_cookies(cookies)
            if exported:
                has_unb = any(c.get("name") == config.login_cookie_name for c in exported)
                has_m_h5_tk = any(c.get("name") == "_m_h5_tk" for c in exported)
                if has_unb and has_m_h5_tk:
                    await session.close()
                    return (
                        True,
                        "登录成功，已提取 cookies",
                        {"status": STATUS_SUCCESS, "cookies": exported},
                    )
        except Exception as error:  # noqa: BLE001
            logger.warning(
                "密码登录导出 cookies 失败（第 %s 次）：%s",
                attempt + 1,
                str(error)[:100],
            )
        await asyncio.sleep(1)

    await session.close()
    return False, "已登录但 cookies 尚未稳定导出，请重试或改用扫码登录", {"status": STATUS_FAILED}


async def password_login(login_id: str, password: str) -> tuple[bool, str, dict[str, Any]]:
    """账号密码登录：在真实 Playwright 浏览器上下文中提交登录表单并导出 cookies。"""
    return await password_login_by_platform(login_id, password, platform="xianyu")


async def password_login_by_platform(
    login_id: str,
    password: str,
    *,
    platform: str = "xianyu",
) -> tuple[bool, str, dict[str, Any]]:
    """按平台执行账号密码登录。

    @param login_id 登录账号
    @param password 登录密码
    @param platform 平台标识（默认 xianyu）
    """
    if async_playwright is None:
        msg = "playwright 未安装：请运行 uv add playwright 并执行 playwright install chromium"
        return False, msg, {"status": STATUS_FAILED}

    platform_name = normalize_platform(platform)
    config = get_platform_config(platform_name)
    session = BrowserSession()
    try:
        user_agent = resolve_user_agent(platform_name)
        proxy = resolve_proxy(platform_name)
        playwright = await async_playwright().start()
        launch_kwargs: dict[str, Any] = {
            "headless": True,
            "args": [
                "--disable-blink-features=AutomationControlled",
                "--disable-dev-shm-usage",
                "--no-sandbox",
                "--disable-setuid-sandbox",
            ],
        }
        if proxy:
            launch_kwargs["proxy"] = proxy
        browser = await playwright.chromium.launch(**launch_kwargs)
        context = await browser.new_context(
            user_agent=user_agent,
            viewport={"width": 1280, "height": 800},
            is_mobile=False,
            locale="zh-CN",
            timezone_id="Asia/Shanghai",
        )
        await apply_stealth(context, logger)
        page = await context.new_page()

        session.browser = browser
        session.context = context
        session.page = page

        await page.goto(config.login_entry_url, wait_until="domcontentloaded", timeout=40000)
        await page.wait_for_timeout(1500)
        await try_click_first(page, config.password_tab_selectors)

        if not await fill_first(page, config.login_id_selectors, login_id):
            raise RuntimeError("未找到账号输入框")
        if not await fill_first(page, config.password_selectors, password):
            raise RuntimeError("未找到密码输入框")
        if not await try_click_first(page, config.submit_selectors):
            raise RuntimeError("未找到登录按钮")

        deadline = time.monotonic() + 20
        while time.monotonic() < deadline:
            page_text = ""
            with contextlib.suppress(Exception):
                page_text = (await page.locator("body").inner_text(timeout=1500))[:400]

            if any(keyword in page_text for keyword in ("滑块", "验证", "短信验证码", "人脸")):
                return False, "密码登录需要人工验证，请改用扫码登录", {"status": STATUS_FAILED}

            error_text = await extract_first_text(page, config.error_selectors)
            if error_text:
                return False, error_text, {"status": STATUS_FAILED}

            cookies = await context.cookies()
            logged_in = any(
                c.get("name") == config.login_cookie_name
                and config.cookie_domain_keyword in str(c.get("domain", ""))
                for c in cookies
            )
            if not logged_in:
                url = page.url.lower()
                logged_in = (
                    config.cookie_domain_keyword in url
                    and "login" not in url
                    and "passport" not in url
                )
            if logged_in:
                return await _finish_password_success(session, platform=platform_name)

            await page.wait_for_timeout(1000)

        await session.close()
        return False, "密码登录超时，请检查账号密码或改用扫码登录", {"status": STATUS_FAILED}
    except Exception as error:  # noqa: BLE001
        await session.close()
        logger.exception("密码登录失败", extra={"event": "channel.password_login.failed"})
        return False, f"密码登录失败: {error}", {"status": STATUS_FAILED}

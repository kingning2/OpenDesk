"""闲鱼 Cookie 浏览器续期：风控 / 验证码时临时启动 Playwright。

对齐 xianyu-auto-reply：
- 稳态不常驻浏览器；
- 风控页优先自动拖滑块；
- 自动失败再回退有头人工完成。

作者：Xiaoman
创建时间：2026-08-19
"""

from __future__ import annotations

import asyncio
import contextlib
import logging
import os
import re
import time
from pathlib import Path
from typing import Any

from gateway.login.platform_config import get_platform_config, normalize_platform
from gateway.login.playwright_common import (
    LAUNCH_ARGS,
    apply_anti_detect,
    apply_stealth,
    async_playwright,
    resolve_proxy,
    resolve_user_agent,
    to_serializable_cookies,
)
from gateway.login.slider_solve import clear_risk_cookies, has_x5sec, try_solve_slider

logger = logging.getLogger("dingda.sidecar.cookie-renew")

STATUS_SUCCESS = "success"
STATUS_FAILED = "failed"
DEFAULT_TIMEOUT_SECS = 180
GOOFISH_DOMAIN = ".goofish.com"


def _safe_account_dir(account_id: str) -> str:
    """把账号 id 收成可作目录名的片段。"""
    cleaned = re.sub(r"[^A-Za-z0-9._-]+", "_", account_id.strip()) or "unknown"
    return cleaned[:80]


def _playwright_cookie(raw: dict[str, Any]) -> dict[str, Any] | None:
    """把契约 Cookie 转成 Playwright add_cookies 入参。"""
    name = str(raw.get("name") or "").strip()
    value = str(raw.get("value") or "").strip()
    if not name or not value:
        return None
    domain = str(raw.get("domain") or "").strip() or GOOFISH_DOMAIN
    if domain.startswith("http"):
        domain = GOOFISH_DOMAIN
    path = str(raw.get("path") or "").strip() or "/"
    cookie: dict[str, Any] = {
        "name": name,
        "value": value,
        "domain": domain,
        "path": path,
    }
    http_only = raw.get("httpOnly")
    if http_only is None:
        http_only = raw.get("http_only")
    if isinstance(http_only, bool):
        cookie["httpOnly"] = http_only
    if isinstance(raw.get("secure"), bool):
        cookie["secure"] = raw["secure"]
    same_site = raw.get("sameSite") or raw.get("same_site")
    if isinstance(same_site, str) and same_site:
        cookie["sameSite"] = same_site
    expires = raw.get("expires")
    if isinstance(expires, (int, float)) and expires > 0:
        cookie["expires"] = expires
    return cookie


def _clear_profile_locks(user_data_dir: Path) -> None:
    """清理上次未干净退出留下的 Chromium 锁文件。"""
    for name in ("SingletonLock", "SingletonCookie", "SingletonSocket"):
        lock = user_data_dir / name
        with contextlib.suppress(OSError):
            if lock.exists():
                lock.unlink()


def _looks_logged_in(page_url: str, cookies: list[dict[str, Any]], login_cookie: str) -> bool:
    """用 Cookie 与当前 URL 判断是否仍保持登录。"""
    has_unb = any(c.get("name") == login_cookie for c in cookies)
    has_tk = any(c.get("name") == "_m_h5_tk" for c in cookies)
    url = page_url.lower()
    blocked = any(token in url for token in ("punish", "captcha", "_____tmd_____", "passport"))
    # 滑块通过后可能仍短暂停在 punish 页，但已有 x5sec
    if has_unb and has_tk and has_x5sec(cookies):
        return True
    return has_unb and has_tk and not blocked


def _want_auto_slider() -> bool:
    """是否启用自动拖滑块（默认开，设 DINGDA_SLIDER_AUTO=0 关闭）。"""
    value = os.getenv("DINGDA_SLIDER_AUTO", "1").strip().lower()
    return value not in {"0", "false", "no", "off"}


def _resolve_headless(*, has_punish: bool, force_headed: bool, try_auto: bool = False) -> bool:
    """解析是否无头。

    自动滑块默认有头（反检测更好、用户可见进度）；
    设 DINGDA_COOKIE_RENEW_HEADLESS=1 可改无头；人工回退始终有头。
    """
    if force_headed:
        return False
    env = os.getenv("DINGDA_COOKIE_RENEW_HEADLESS", "").strip().lower()
    if env in {"0", "false", "no", "off"}:
        return False
    if env in {"1", "true", "yes", "on"}:
        return True
    if try_auto and _want_auto_slider():
        return False
    return bool(has_punish and _want_auto_slider())


async def _run_renew_session(
    *,
    prepared: list[dict[str, Any]],
    account_id: str,
    target: str,
    punish: str,
    platform_name: str,
    config: Any,
    timeout_secs: int,
    headless: bool,
    try_auto: bool,
) -> tuple[bool, str, dict[str, Any]]:
    """启动一次浏览器会话并尝试续期。

    @returns (成功, 说明, {status, cookies})
    """
    user_data_dir = Path.cwd() / "browser_data" / f"user_{_safe_account_dir(account_id)}"
    user_data_dir.mkdir(parents=True, exist_ok=True)
    _clear_profile_locks(user_data_dir)

    playwright = None
    context = None
    try:
        user_agent = resolve_user_agent(platform_name)
        proxy = resolve_proxy(platform_name)
        playwright = await async_playwright().start()
        launch_kwargs: dict[str, Any] = {
            "headless": headless,
            "args": list(LAUNCH_ARGS),
            # 去掉 Playwright 默认 --enable-automation，显著降低"自动化窗口"被标记概率
            "ignore_default_args": ["--enable-automation"],
            "viewport": {"width": 1280, "height": 720},
            "user_agent": user_agent,
            "locale": "zh-CN",
            "timezone_id": "Asia/Shanghai",
        }
        if proxy:
            launch_kwargs["proxy"] = proxy
        context = await playwright.chromium.launch_persistent_context(
            str(user_data_dir),
            **launch_kwargs,
        )
        await apply_stealth(context, logger)
        await apply_anti_detect(context, logger)
        with contextlib.suppress(Exception):
            await context.add_cookies(prepared)

        page = context.pages[0] if context.pages else await context.new_page()
        logger.info(
            "开始浏览器续期 account=%s headless=%s auto_slider=%s url=%s",
            account_id,
            headless,
            try_auto,
            target[:120],
            extra={"event": "channel.cookie_renew.started", "account_id": account_id},
        )
        await page.goto(target, wait_until="domcontentloaded", timeout=40000)
        await page.wait_for_timeout(1500)
        # 清掉历史 risk cookies，让风控挑战重新开始，避免复用旧的 punish 态
        await clear_risk_cookies(context)

        if try_auto:
            ok, detail = await try_solve_slider(page, context, max_retries=3)
            logger.info(
                "自动滑块结果 account=%s ok=%s detail=%s",
                account_id,
                ok,
                detail,
            )
            if ok:
                await page.wait_for_timeout(800)
                with contextlib.suppress(Exception):
                    await page.goto(
                        config.home_url,
                        wait_until="domcontentloaded",
                        timeout=40000,
                    )
                    await page.wait_for_timeout(1500)
                exported = to_serializable_cookies(await context.cookies())
                if _looks_logged_in(page.url, exported, config.login_cookie_name) or has_x5sec(
                    exported
                ):
                    return (
                        True,
                        "自动滑块通过，已导出 Cookie",
                        {"status": STATUS_SUCCESS, "cookies": exported},
                    )

        # 人工等待（有头）或自动失败后的兜底轮询
        deadline = time.monotonic() + max(15, timeout_secs)
        last_detail = "等待页面登录态稳定"
        if punish and not headless:
            last_detail = "请在弹出的浏览器窗口完成滑块验证"
            logger.info(
                "等待人工完成滑块 account=%s",
                account_id,
            )

        while time.monotonic() < deadline:
            exported = to_serializable_cookies(await context.cookies())
            url = page.url
            if _looks_logged_in(url, exported, config.login_cookie_name):
                if ("punish" in url.lower() or "captcha" in url.lower()) and not has_x5sec(
                    exported
                ):
                    last_detail = "仍在验证码页，请完成滑块"
                    await page.wait_for_timeout(1500)
                    continue
                if "goofish.com" not in url.lower():
                    with contextlib.suppress(Exception):
                        await page.goto(
                            config.home_url,
                            wait_until="domcontentloaded",
                            timeout=40000,
                        )
                        await page.wait_for_timeout(2000)
                    exported = to_serializable_cookies(await context.cookies())
                logger.info(
                    "浏览器续期成功 account=%s cookies=%s",
                    account_id,
                    len(exported),
                    extra={"event": "channel.cookie_renew.completed", "account_id": account_id},
                )
                return (
                    True,
                    "浏览器续期成功，已导出 Cookie",
                    {"status": STATUS_SUCCESS, "cookies": exported},
                )
            last_detail = "请在弹出的浏览器窗口完成验证码或登录"
            await page.wait_for_timeout(1500)

        return False, f"浏览器续期超时：{last_detail}", {"status": STATUS_FAILED}
    except Exception as error:  # noqa: BLE001
        logger.exception(
            "浏览器续期失败 account=%s",
            account_id,
            extra={"event": "channel.cookie_renew.failed", "account_id": account_id},
        )
        return False, f"浏览器续期失败: {error}", {"status": STATUS_FAILED}
    finally:
        if context is not None:
            with contextlib.suppress(Exception):
                await context.close()
        if playwright is not None:
            with contextlib.suppress(Exception):
                await playwright.stop()
        await asyncio.sleep(0.05)


async def renew_cookies(
    cookies: list[dict[str, Any]],
    *,
    account_id: str,
    punish_url: str | None = None,
    platform: str = "xianyu",
    timeout_secs: int = DEFAULT_TIMEOUT_SECS,
) -> tuple[bool, str, dict[str, Any]]:
    """注入现有 Cookie，优先自动过滑块，失败再有头人工。

    @param cookies 现有 Cookie 列表（契约 ChannelCookie）
    @param account_id 账号 id（用于持久化目录）
    @param punish_url 风控惩罚页；空则打开闲鱼首页
    @param platform 平台标识
    @param timeout_secs 最长等待秒数（人工回退阶段）
    @returns (成功, 说明, {status, cookies})
    """
    if async_playwright is None:
        msg = "playwright 未安装：请运行 uv add playwright 并执行 playwright install chromium"
        return False, msg, {"status": STATUS_FAILED}

    platform_name = normalize_platform(platform)
    config = get_platform_config(platform_name)
    prepared = [item for item in (_playwright_cookie(raw) for raw in cookies) if item]
    if not prepared:
        return False, "没有可注入的 Cookie，请先扫码或密码登录", {"status": STATUS_FAILED}

    punish = (punish_url or "").strip()
    target = punish or config.home_url
    want_auto = _want_auto_slider()

    # 1) 自动滑块（默认有头；不强制要求 punish URL）
    if want_auto:
        headless = _resolve_headless(has_punish=bool(punish), force_headed=False, try_auto=True)
        ok, detail, data = await _run_renew_session(
            prepared=prepared,
            account_id=account_id,
            target=target,
            punish=punish,
            platform_name=platform_name,
            config=config,
            timeout_secs=min(60, timeout_secs),
            headless=headless,
            try_auto=True,
        )
        if ok:
            return ok, detail, data
        logger.warning(
            "自动滑块未通过，回退有头人工 account=%s detail=%s",
            account_id,
            detail,
        )

    # 2) 有头人工 / 关闭自动时的普通续期
    headless = _resolve_headless(has_punish=bool(punish), force_headed=True, try_auto=False)
    return await _run_renew_session(
        prepared=prepared,
        account_id=account_id,
        target=target,
        punish=punish,
        platform_name=platform_name,
        config=config,
        timeout_secs=timeout_secs,
        headless=headless,
        try_auto=False,
    )

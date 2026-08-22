"""扫码登录会话管理（qrcode）— 按平台配置生成二维码并轮询登录状态。

作者：Xiaoman
创建时间：2026-08-18
"""

from __future__ import annotations

import asyncio
import base64
import contextlib
import logging
import os
import time
import uuid
from pathlib import Path
from typing import Any

from gateway.platform_config import get_platform_config, normalize_platform
from gateway.playwright_common import (
    LAUNCH_ARGS,
    apply_anti_detect,
    apply_stealth,
    async_playwright,
    launch_persistent_chromium,
    resolve_proxy,
    to_serializable_cookies,
)

logger = logging.getLogger("dingda.sidecar.qr")

QR_WAIT_TIMEOUT_MS = 15000

# 扫码状态机。
STATUS_GENERATING = "generating"
STATUS_READY = "ready"
STATUS_WAITING = "waiting"
STATUS_SCANNED = "scanned"
STATUS_CONFIRMED = "confirmed"
STATUS_SUCCESS = "success"
STATUS_REFRESHED = "refreshed"
STATUS_EXPIRED = "expired"
STATUS_FAILED = "failed"

# 二维码刷新周期（秒）：二维码寿命短，到点原地刷新保持可扫。
QR_REFRESH_SECONDS = 30
# 整个扫码尝试的硬上限（秒）：超过仍未登录则结束会话。
QR_EXPIRE_SECONDS = 300


class QrSession:
    """一次扫码登录的 Playwright 会话。"""

    def __init__(self, session_id: str, platform: str) -> None:
        self.session_id = session_id
        self.platform = platform
        self.browser: Any | None = None
        self.context: Any | None = None
        self.page: Any | None = None
        self.status = STATUS_GENERATING
        self.started_at = time.monotonic()
        self.last_refresh_at = time.monotonic()
        self.qr_base64: str | None = None
        self.detail = ""
        # 串行化对该会话的操作：共享事件循环下并发轮询会损坏 Playwright 上下文。
        self.lock = asyncio.Lock()

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


def _to_base64(screenshot: bytes) -> str:
    """截图字节 → data URL。"""
    return "data:image/png;base64," + base64.b64encode(screenshot).decode("ascii")


async def start_qr_login() -> tuple[bool, str, dict[str, Any]]:
    """启动扫码登录：打开登录页，截图二维码。返回 (ok, detail, payload)。"""
    return await start_qr_login_by_platform(platform="xianyu")


def _qr_headless() -> bool:
    """扫码是否无头。默认无头；设 DINGDA_QR_HEADLESS=0 可弹出浏览器调试。"""
    value = os.getenv("DINGDA_QR_HEADLESS", "1").strip().lower()
    return value not in {"0", "false", "no", "off"}


async def start_qr_login_by_platform(platform: str = "xianyu") -> tuple[bool, str, dict[str, Any]]:
    """按平台启动扫码登录。"""
    if async_playwright is None:
        msg = "playwright 未安装：请运行 uv add playwright 并执行 playwright install chromium"
        return False, msg, {"status": STATUS_FAILED}

    platform_name = normalize_platform(platform)
    config = get_platform_config(platform_name)
    qr_locator = ", ".join(config.qr_selectors)
    session_id = str(uuid.uuid4())
    session = QrSession(session_id, platform_name)

    try:
        proxy = resolve_proxy(platform_name)
        headless = _qr_headless()
        logger.info("启动 Playwright 引擎…")
        playwright = await async_playwright().start()
        logger.info(
            "正在打开浏览器（系统 Edge/Chrome，%s）… platform=%s proxy_enabled=%s",
            "无头" if headless else "有头",
            platform_name,
            bool(proxy),
        )
        launch_kwargs: dict[str, Any] = {
            "headless": headless,
            "args": list(LAUNCH_ARGS),
            # 去掉 Playwright 默认 --enable-automation，降低"自动化窗口"被标记概率
            "ignore_default_args": ["--enable-automation"],
            "viewport": {"width": 1280, "height": 800},
            "locale": "zh-CN",
            "timezone_id": "Asia/Shanghai",
        }
        if proxy:
            launch_kwargs["proxy"] = proxy
        user_data_dir = Path.cwd() / "browser_data" / f"qr_{platform_name}"
        context = await launch_persistent_chromium(playwright, user_data_dir, **launch_kwargs)
        await apply_stealth(context, logger)
        await apply_anti_detect(context, logger)
        # 清掉持久化 profile 里可能残留的登录 cookie，确保每次展示登录二维码
        with contextlib.suppress(Exception):
            await context.clear_cookies()
        page = context.pages[0] if context.pages else await context.new_page()

        session.browser = None  # 持久化上下文自带浏览器，由 context.close() 回收
        session.context = context
        session.page = page
        session.status = STATUS_GENERATING

        # 登录页加载：慢网络/风控限流下可能超时，加大超时并重试。
        logger.info("正在加载登录页… platform=%s", platform_name)
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
            raise RuntimeError("闲鱼登录页加载失败")

        # 等二维码元素出现：多选择器合并一次等可见，命中即返回，最坏 15s。
        qr_element = None
        try:
            qr_element = await page.wait_for_selector(
                qr_locator, state="visible", timeout=QR_WAIT_TIMEOUT_MS
            )
        except Exception:  # noqa: BLE001
            qr_element = None

        if qr_element is None:
            # 兜底：截图整个页面区域（可能二维码不是 img 元素）。
            fallback: bytes | None = None
            with contextlib.suppress(Exception):
                await page.wait_for_timeout(1500)
                fallback = await page.screenshot()
            if not fallback:
                raise RuntimeError("未检测到二维码元素")
            session.status = STATUS_READY
            session.detail = "已显示登录页（兜底截图）"
            session.qr_base64 = _to_base64(fallback)
            QR_SESSIONS[session_id] = session
            return (
                True,
                "已显示登录页",
                {
                    "status": STATUS_READY,
                    "session_id": session_id,
                    "qr_base64": session.qr_base64,
                },
            )

        screenshot = await qr_element.screenshot()
        session.status = STATUS_READY
        session.detail = "二维码已就绪"
        session.qr_base64 = _to_base64(screenshot)
        QR_SESSIONS[session_id] = session
        logger.info("二维码已就绪，可扫码登录")
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


async def _refresh_qr(session: QrSession) -> str | None:
    """原地刷新二维码：重新加载登录页，等新二维码渲染后截图。失败时记录原因。"""
    config = get_platform_config(session.platform)
    qr_locator = ", ".join(config.qr_selectors)
    page = session.page
    if page is None:
        logger.warning("二维码刷新失败：浏览器会话已关闭")
        return None
    try:
        await page.goto(config.login_entry_url, wait_until="domcontentloaded", timeout=40000)
        element = await page.wait_for_selector(qr_locator, state="visible", timeout=15000)
        screenshot = await element.screenshot()
        return _to_base64(screenshot)
    except Exception as error:  # noqa: BLE001
        logger.warning(
            "二维码刷新失败：%s",
            error,
            extra={"event": "channel.qr.refresh_failed", "reason": str(error)},
        )
        return None


async def _finish_success(session: QrSession, session_id: str) -> tuple[bool, str, dict[str, Any]]:
    """登录成功：先访问闲鱼首页建立 h5 mtop 会话（_m_h5_tk），再导出 cookies。

    导出失败/为空时重试；**`_m_h5_tk` 缺失时补访首页再导出**（缺失会导致后续
    Rust 侧 mtop 签名 token 为空 → 被风控拦截）。持续失败则返回确认态让下一轮再试，不破坏会话。
    """
    config = get_platform_config(session.platform)
    page = session.page
    if page is not None:
        with contextlib.suppress(Exception):
            url = page.url.lower()
            # 已登录的平台页面（非 passport）则跳过。
            if not (config.cookie_domain_keyword in url and "passport" not in url):
                await page.goto(config.home_url, wait_until="domcontentloaded", timeout=40000)
                # 等首页 h5 mtop 调用落盘 _m_h5_tk 等会话 cookie。
                await page.wait_for_timeout(3000)

    for attempt in range(3):
        try:
            cookies = await session.context.cookies()
            exported = to_serializable_cookies(cookies)
            if exported:
                has_m_h5_tk = any(c.get("name") == "_m_h5_tk" for c in exported)
                if not has_m_h5_tk:
                    # 签名 token 缺失：补访首页触发 mtop 落盘后重试（最多补 1 次）。
                    logger.warning(
                        "导出 cookies 缺少 _m_h5_tk，补访首页触发 mtop 会话（第 %s 次）",
                        attempt + 1,
                    )
                    if attempt == 0 and page is not None:
                        with contextlib.suppress(Exception):
                            await page.goto(
                                config.home_url,
                                wait_until="domcontentloaded",
                                timeout=40000,
                            )
                            await page.wait_for_timeout(3000)
                        continue
                await session.close()
                QR_SESSIONS.pop(session_id, None)
                return (
                    True,
                    "登录成功，已提取 cookies",
                    {"status": STATUS_SUCCESS, "cookies": exported},
                )
            logger.warning("cookies 为空，重试导出（第 %s 次）", attempt + 1)
        except Exception as error:  # noqa: BLE001
            logger.warning("导出 cookies 失败（第 %s 次）：%s", attempt + 1, str(error)[:100])
        await asyncio.sleep(1)

    # 多次失败：返回确认态，下一轮轮询再试。
    return True, "已确认登录，正在提取 cookies", {"status": STATUS_CONFIRMED}


async def check_qr_login(session_id: str) -> tuple[bool, str, dict[str, Any]]:
    """轮询扫码状态。已扫码/登录成功优先判定，二维码仅在纯等待态刷新。"""
    return await check_qr_login_by_platform(session_id, platform="xianyu")


async def check_qr_login_by_platform(
    session_id: str,
    *,
    platform: str = "xianyu",
) -> tuple[bool, str, dict[str, Any]]:
    """按平台轮询扫码状态。"""
    platform_name = normalize_platform(platform)
    session = QR_SESSIONS.get(session_id)
    if session is None:
        return False, "会话不存在或已过期", {"status": STATUS_FAILED}
    if session.platform != platform_name:
        return False, "平台与会话不匹配", {"status": STATUS_FAILED}

    # 串行化：共享事件循环下并发轮询会损坏 Playwright 上下文。
    async with session.lock:
        # 整个扫码尝试的硬上限：超过仍未登录则结束。
        if time.monotonic() - session.started_at > QR_EXPIRE_SECONDS:
            await session.close()
            QR_SESSIONS.pop(session_id, None)
            return True, "二维码已过期", {"status": STATUS_EXPIRED}

        try:
            page = session.page
            if page is None:
                return True, "等待中", {"status": STATUS_WAITING}

            url = page.url.lower()
            config = get_platform_config(session.platform)

            # 1. 登录成功：URL 回到平台域且不含登录跳转，或平台域已有登录 cookie。
            logged_in = (
                config.cookie_domain_keyword in url and "login" not in url and "passport" not in url
            )
            if not logged_in:
                # passport 页登录后可能不自动跳转；用平台域登录 cookie（如 unb）兜底。
                # 登录成功后才写入）兜底判断。注意不能用 _tb_token_/cookie2，登录页也会写入。
                with contextlib.suppress(Exception):
                    cookies = await session.context.cookies()
                    logged_in = any(
                        c.get("name") == config.login_cookie_name
                        and config.cookie_domain_keyword in str(c.get("domain", ""))
                        for c in cookies
                    )

            if logged_in:
                return await _finish_success(session, session_id)

            # 2. DOM 文案检测。注意页面用词是"扫码成功"而非"扫描成功"。
            page_text = ""
            with contextlib.suppress(Exception):
                page_text = (await page.locator("body").inner_text(timeout=2000))[:300]

            if "确认" in page_text and (
                "已扫码" in page_text or "扫码成功" in page_text or "扫描成功" in page_text
            ):
                return True, "已扫码，请在手机确认", {"status": STATUS_SCANNED}
            if (
                "确认登录" in page_text
                or "请在手机" in page_text
                or ("确认" in page_text and "二维码" in page_text)
            ):
                return True, "已扫码，请在手机确认", {"status": STATUS_CONFIRMED}
            # passport 登录成功态文案（登录后未跳转时）。
            if "登录成功" in page_text or "欢迎" in page_text:
                return await _finish_success(session, session_id)

            # 3. 刷新：仅在真正等待扫码时（到刷新周期或二维码元素消失）。
            qr_missing = False
            with contextlib.suppress(Exception):
                qr_missing = await page.locator(config.qr_selectors[0]).count() == 0
            if qr_missing or time.monotonic() - session.last_refresh_at > QR_REFRESH_SECONDS:
                new_qr = await _refresh_qr(session)
                if new_qr is not None:
                    session.qr_base64 = new_qr
                    session.last_refresh_at = time.monotonic()
                    return True, "二维码已刷新", {"status": STATUS_REFRESHED, "qr_base64": new_qr}
                return True, "二维码已过期，刷新失败", {"status": STATUS_EXPIRED}

            return True, "等待扫码", {"status": STATUS_WAITING}

        except Exception as error:  # noqa: BLE001
            await session.close()
            QR_SESSIONS.pop(session_id, None)
            logger.exception("扫码状态检查失败", extra={"event": "channel.qr.check_failed"})
            return False, f"扫码状态检查失败: {error}", {"status": STATUS_FAILED}


async def cancel_qr_login(session_id: str) -> tuple[bool, str, dict[str, Any]]:
    """取消扫码登录：关闭浏览器，清理会话。"""
    return await cancel_qr_login_by_platform(session_id, platform="xianyu")


async def cancel_qr_login_by_platform(
    session_id: str,
    *,
    platform: str = "xianyu",
) -> tuple[bool, str, dict[str, Any]]:
    """按平台取消扫码登录。"""
    platform_name = normalize_platform(platform)
    session = QR_SESSIONS.pop(session_id, None)
    if session is None:
        return True, "会话不存在", {"ok": True}
    if session.platform != platform_name:
        QR_SESSIONS[session_id] = session
        return False, "平台与会话不匹配", {"ok": False}
    await session.close()
    return True, "已取消", {"ok": True}

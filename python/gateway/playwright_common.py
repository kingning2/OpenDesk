"""Playwright 登录公共能力：浏览器启动、UA/代理、stealth、通用页面操作。

作者：Xiaoman
创建时间：2026-08-18
"""

from __future__ import annotations

import contextlib
import logging
import os
from pathlib import Path
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

logger = logging.getLogger("dingda.sidecar.playwright")

# 系统浏览器优先（避免下载 Playwright Chromium；真实 Edge/Chrome 指纹更抗风控）。
SYSTEM_CHANNELS = ("msedge", "chrome")

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
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-popup-blocking",
]

# 闲鱼 / 阿里 Baxia 风控在刷新重试前必须清除的 risk cookies。
# 若不清理会形成"刷新 → 带 risk cookies → 再次 punish → 刷新"死循环。
RISK_COOKIE_NAMES = (
    "x5secdata",
    "x5sec",
    "x5sectag",
    "x5pref",
    "bx-cookie-test",
    "tfstk",
    "cbc",
    "sca",
    "isg",
)

# 注入到每个页面的反检测脚本：覆盖 FireyeJS / Baxia 常用自动化指纹检测点。
# playwright-stealth 已覆盖 webdriver/plugins/chrome/languages 等，这里补齐
# stealth 未处理的：platform/vendor 与 UA 一致性、userAgentData、WebGL 渲染器、
# Canvas 指纹微扰动、CDP 注入痕迹清理。参考 XianYuPilo sliderSolver.ts。
ANTI_DETECT_SCRIPT = r"""
(() => {
  try {
    const NAV = navigator;
    const DEF = (target, name, value) => {
      try {
        Object.defineProperty(target, name, { get: () => value, configurable: true });
      } catch (e) {}
    };
    // platform/vendor 与 UA 保持一致，避免 FireyeJS 判定为虚拟机/服务器
    DEF(Navigator.prototype, 'platform', 'Win32');
    DEF(NAV, 'platform', 'Win32');
    DEF(NAV, 'vendor', 'Google Inc.');
    const ua = NAV.userAgent || '';
    if (ua) DEF(NAV, 'appVersion', ua.replace(/^Mozilla\//, ''));
    DEF(NAV, 'hardwareConcurrency', 8);
    DEF(NAV, 'deviceMemory', 8);
    // userAgentData（Client Hints）：覆盖真实 OS 指纹，FireyeJS 依赖它
    const ver = (ua.match(/Chrome\/([\d]+)/) || [, '146'])[1];
    const brands = [
      { brand: 'Google Chrome', version: ver },
      { brand: 'Chromium', version: ver },
      { brand: 'Not.A/Brand', version: '8' },
    ];
    DEF(NAV, 'userAgentData', {
      brands,
      mobile: false,
      platform: 'Windows',
      getHighEntropyValues: (hints) => Promise.resolve({
        architecture: 'x86', bitness: '64', brands,
        fullVersionList: brands, mobile: false, model: '',
        platform: 'Windows', platformVersion: '15.0.0',
        uaFullVersion: ver, wow64: false,
      }),
      toJSON: () => ({ brands, mobile: false, platform: 'Windows' }),
    });
    // WebGL 渲染器：headless/虚拟机返回 SwiftShader 是强机器人信号
    const wrapWebGL = (Ctx) => {
      if (!Ctx) return;
      const orig = Ctx.prototype.getParameter;
      Ctx.prototype.getParameter = function (param) {
        if (param === 0x9245) return 'Google Inc. (NVIDIA)';
        if (param === 0x9246) {
          return 'ANGLE (NVIDIA, NVIDIA GeForce GTX 1060 Direct3D11 vs_5_0 ps_5_0)';
        }
        return orig.call(this, param);
      };
    };
    wrapWebGL(WebGLRenderingContext);
    if (typeof WebGL2RenderingContext !== 'undefined') wrapWebGL(WebGL2RenderingContext);
    // Canvas 指纹微扰动：在 toDataURL 结果中注入 ±1 噪声，改变指纹哈希
    const origData = HTMLCanvasElement.prototype.toDataURL;
    HTMLCanvasElement.prototype.toDataURL = function (...args) {
      const ctx = this.getContext('2d');
      if (ctx) {
        try {
          const w = this.width, h = this.height;
          if (w > 0 && h > 0) {
            const img = ctx.getImageData(0, 0, w, h);
            for (let i = 0; i < img.data.length; i += 4) {
              if (Math.random() < 0.03) {
                img.data[i] = (img.data[i] + (Math.random() < 0.5 ? -1 : 1)) & 0xff;
              }
            }
            ctx.putImageData(img, 0, 0);
          }
        } catch (e) {}
      }
      return origData.apply(this, args);
    };
    // 清理 CDP 注入痕迹
    for (const key of Object.keys(window)) {
      if (key.startsWith('cdc_')) { try { delete window[key]; } catch (e) {} }
    }
  } catch (e) {}
})();
"""


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


async def apply_anti_detect(context: BrowserContext, logger: Any) -> None:
    """注入补充反检测脚本，覆盖 stealth 未处理的指纹检测点。"""
    try:
        await context.add_init_script(ANTI_DETECT_SCRIPT)
    except Exception:  # noqa: BLE001
        logger.warning("注入反检测脚本失败（不影响主流程）")


def clear_profile_locks(user_data_dir: Path) -> None:
    """清理上次未干净退出留下的 Chromium 锁文件。"""
    for name in ("SingletonLock", "SingletonCookie", "SingletonSocket"):
        lock = user_data_dir / name
        with contextlib.suppress(OSError):
            if lock.exists():
                lock.unlink()


async def launch_persistent_chromium(
    playwright: Any,
    user_data_dir: str | Path,
    **kwargs: Any,
) -> BrowserContext:
    """用系统浏览器（Edge/Chrome）启动持久化上下文；都不可用则退回自带 Chromium。

    避免为登录/续期下载 Playwright Chromium；真实 Edge/Chrome 指纹更抗风控。
    """
    clear_profile_locks(Path(user_data_dir))
    last_error: str | None = None
    for channel in SYSTEM_CHANNELS + (None,):
        opts = dict(kwargs)
        if channel:
            opts["channel"] = channel
        try:
            context = await playwright.chromium.launch_persistent_context(
                str(user_data_dir), **opts
            )
            logger.info("浏览器已启动 channel=%s", channel or "bundled")
            return context
        except Exception as error:  # noqa: BLE001
            last_error = str(error)
            logger.warning("浏览器启动失败 channel=%s: %s", channel, str(error)[:120])
    raise RuntimeError(f"启动浏览器失败（已尝试 Edge/Chrome/自带）: {last_error}")


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

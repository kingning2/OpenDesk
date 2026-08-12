"""浏览器快照恢复辅助 — 移植自 ai-goofish-monitor。

将 Chrome 扩展导出的完整浏览器快照（cookies + env + storage + headers）
应用到 Playwright 上下文，尽量还原真实浏览器指纹以规避风控。
"""

from __future__ import annotations

import json
from typing import Any


def default_context_options() -> dict[str, Any]:
    """无快照时的移动端默认上下文参数。"""
    return {
        "user_agent": (
            "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) "
            "AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Mobile Safari/537.36"
        ),
        "viewport": {"width": 412, "height": 915},
        "device_scale_factor": 2.625,
        "is_mobile": True,
        "has_touch": True,
        "locale": "zh-CN",
        "timezone_id": "Asia/Shanghai",
        "permissions": ["geolocation"],
        "geolocation": {"longitude": 121.4737, "latitude": 31.2304},
        "color_scheme": "light",
    }


def clean_kwargs(options: dict[str, Any]) -> dict[str, Any]:
    return {key: value for key, value in options.items() if value is not None}


def _looks_like_mobile(ua: str) -> bool | None:
    if not ua:
        return None
    ua_lower = ua.lower()
    if "mobile" in ua_lower or "android" in ua_lower or "iphone" in ua_lower:
        return True
    if "windows" in ua_lower or "macintosh" in ua_lower:
        return False
    return None


def build_context_overrides(snapshot: dict[str, Any]) -> dict[str, Any]:
    """从快照的 env/headers 提取上下文覆盖参数。"""
    env = snapshot.get("env") or {}
    headers = snapshot.get("headers") or {}
    navigator = env.get("navigator") or {}
    screen = env.get("screen") or {}
    intl = env.get("intl") or {}

    overrides: dict[str, Any] = {}

    ua = headers.get("User-Agent") or headers.get("user-agent") or navigator.get("userAgent")
    if ua:
        overrides["user_agent"] = ua

    accept_language = headers.get("Accept-Language") or headers.get("accept-language")
    locale = None
    if accept_language:
        locale = str(accept_language).split(",")[0].strip()
    elif navigator.get("language"):
        locale = navigator["language"]
    if locale:
        overrides["locale"] = locale

    tz = intl.get("timeZone")
    if tz:
        overrides["timezone_id"] = tz

    width = screen.get("width")
    height = screen.get("height")
    if isinstance(width, (int, float)) and isinstance(height, (int, float)):
        overrides["viewport"] = {"width": int(width), "height": int(height)}

    dpr = screen.get("devicePixelRatio")
    if isinstance(dpr, (int, float)):
        overrides["device_scale_factor"] = float(dpr)

    touch_points = navigator.get("maxTouchPoints")
    if isinstance(touch_points, (int, float)):
        overrides["has_touch"] = touch_points > 0

    mobile_flag = _looks_like_mobile(ua or "")
    if mobile_flag is not None:
        overrides["is_mobile"] = mobile_flag

    return clean_kwargs(overrides)


def build_extra_headers(raw_headers: dict[str, Any] | None) -> dict[str, str]:
    """从快照 headers 提取附加请求头（排除 cookie/content-length）。"""
    if not raw_headers:
        return {}
    excluded = {"cookie", "content-length"}
    headers: dict[str, str] = {}
    for key, value in raw_headers.items():
        if not key or key.lower() in excluded or value is None:
            continue
        headers[str(key)] = str(value)
    return headers


def build_storage_state(snapshot: dict[str, Any]) -> dict[str, Any]:
    """构建 Playwright storage_state：仅取快照 cookies。"""
    return {"cookies": snapshot.get("cookies") or []}


ANTI_DETECT_SCRIPT = """
// 移除webdriver标识
Object.defineProperty(navigator, 'webdriver', {get: () => undefined});

// 模拟真实移动设备的navigator属性
Object.defineProperty(navigator, 'plugins', {get: () => [1, 2, 3, 4, 5]});
Object.defineProperty(navigator, 'languages', {get: () => ['zh-CN', 'zh', 'en-US', 'en']});

// 添加chrome对象
window.chrome = {runtime: {}, loadTimes: function() {}, csi: function() {}};

// 模拟触摸支持
Object.defineProperty(navigator, 'maxTouchPoints', {get: () => 5});

// 覆盖permissions查询（避免暴露自动化）
const originalQuery = window.navigator.permissions.query;
window.navigator.permissions.query = (parameters) => (
    parameters.name === 'notifications' ?
        Promise.resolve({state: Notification.permission}) :
        originalQuery(parameters)
);
"""


def parse_snapshot(credential: str) -> dict[str, Any]:
    """解析账号凭据为快照字典。

    兼容两种形态：
    - 快照 JSON（Chrome 扩展导出，含 env/headers/cookies）
    - 旧 cookie 字符串（无结构化信息时返回空 dict）
    """
    try:
        parsed = json.loads(credential)
        if isinstance(parsed, dict):
            return parsed
    except json.JSONDecodeError:
        pass
    return {}

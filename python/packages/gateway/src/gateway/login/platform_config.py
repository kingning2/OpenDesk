"""登录平台配置中心：按平台区分登录页与选择器。

作者：Xiaoman
创建时间：2026-08-18
"""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class LoginPlatformConfig:
    """单个平台的登录配置。"""

    login_entry_url: str
    home_url: str
    qr_selectors: list[str]
    password_tab_selectors: list[str]
    login_id_selectors: list[str]
    password_selectors: list[str]
    submit_selectors: list[str]
    error_selectors: list[str]
    login_cookie_name: str
    cookie_domain_keyword: str


PLATFORM_CONFIGS: dict[str, LoginPlatformConfig] = {
    "xianyu": LoginPlatformConfig(
        login_entry_url=(
            "https://passport.goofish.com/mini_login.htm"
            "?lang=zh_cn&appName=xianyu&appEntrance=web&styleType=vertical"
            "&isMobile=false&qrCodeFirst=true&notKeepLogin=false"
        ),
        home_url="https://www.goofish.com/",
        qr_selectors=[
            "div.qrcode-img",
            "#qrcode-img",
            "canvas",
            "img.qrcode-img",
        ],
        password_tab_selectors=[
            'text="账号密码登录"',
            'text="密码登录"',
            'text="账号登录"',
            '[data-view="password"]',
        ],
        login_id_selectors=[
            'input[name="fm-login-id"]',
            'input[name="loginId"]',
            'input[type="text"]',
            'input[type="tel"]',
        ],
        password_selectors=[
            'input[name="fm-login-password"]',
            'input[name="password"]',
            'input[type="password"]',
        ],
        submit_selectors=[
            'button[type="submit"]',
            'text="登录"',
            'text="立即登录"',
            ".fm-button.fm-submit.password-login",
        ],
        error_selectors=[
            ".fm-error-tip",
            ".fm-error-msg",
            ".error-tip",
        ],
        login_cookie_name="unb",
        cookie_domain_keyword="goofish.com",
    ),
}


def normalize_platform(platform: str | None) -> str:
    """标准化平台标识，默认回退 xianyu。"""
    value = (platform or "xianyu").strip().lower()
    return value or "xianyu"


def get_platform_config(platform: str | None) -> LoginPlatformConfig:
    """获取平台配置；未知平台返回明确错误，避免错误套用闲鱼选择器。"""
    normalized = normalize_platform(platform)
    config = PLATFORM_CONFIGS.get(normalized)
    if config is None:
        supported = ", ".join(sorted(PLATFORM_CONFIGS.keys()))
        raise ValueError(f"不支持的平台: {normalized}（支持: {supported}）")
    return config

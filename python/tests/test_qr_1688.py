"""平台登录 Cookie 判定纯函数测试（不启动 Playwright）。"""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

PYTHON_ROOT = Path(__file__).resolve().parents[1]
if str(PYTHON_ROOT) not in sys.path:
    sys.path.insert(0, str(PYTHON_ROOT))

from channels.core.login.helpers import (  # noqa: E402
    cookie_domains,
    cookies_indicate_platform_login,
    has_login_cookie,
    login_progress_from_text,
)


class CookiePlatformHelpersTests(unittest.TestCase):
    """作者：Xiaoman；创建时间：2026-08-22。"""

    def test_has_login_cookie_platform_scoped(self) -> None:
        cookies = [
            {"name": "unb", "value": "1", "domain": ".taobao.com"},
            {"name": "unb", "value": "2", "domain": ".1688.com"},
        ]
        self.assertTrue(has_login_cookie(cookies, name="unb", domain_keyword="1688.com"))
        self.assertFalse(has_login_cookie(cookies, name="unb", domain_keyword="goofish.com"))

    def test_cookie_domains_sorted_unique(self) -> None:
        cookies = [
            {"name": "a", "value": "1", "domain": ".1688.com"},
            {"name": "b", "value": "2", "domain": ".1688.com"},
            {"name": "c", "value": "3", "domain": ".goofish.com"},
        ]
        self.assertEqual(cookie_domains(cookies), [".1688.com", ".goofish.com"])

    def test_1688_accepts_taobao_unb_as_sso_hint(self) -> None:
        cookies = [{"name": "unb", "value": "1", "domain": ".taobao.com"}]
        self.assertTrue(
            cookies_indicate_platform_login(
                cookies,
                platform="ali1688",
                login_cookie_name="unb",
                domain_keyword="1688.com",
            )
        )
        self.assertFalse(
            cookies_indicate_platform_login(
                cookies,
                platform="xianyu",
                login_cookie_name="unb",
                domain_keyword="goofish.com",
            )
        )


class LoginProgressTextTests(unittest.TestCase):
    """作者：Xiaoman；创建时间：2026-08-22。"""

    def test_waiting_qr_instruction_is_not_confirmed(self) -> None:
        self.assertIsNone(login_progress_from_text("请使用淘宝APP扫描二维码登录"))

    def test_iframe_scanned_copy(self) -> None:
        self.assertEqual(
            login_progress_from_text("扫码成功，请在手机上确认登录"),
            "scanned",
        )

    def test_confirm_on_phone(self) -> None:
        self.assertEqual(login_progress_from_text("请在手机上确认登录"), "confirmed")


if __name__ == "__main__":
    unittest.main()

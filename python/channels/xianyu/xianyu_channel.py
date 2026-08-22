"""闲鱼渠道（参考 CowAgent channel/weixin/weixin_channel.py 组织方式）。"""

from __future__ import annotations

from typing import Any

from channels.channel import Channel
from channels.core.browser.base import BrowserPlatform
from channels.core.login.qrcode import QrcodeLogin
from channels.xianyu.browser import XianyuBrowser
from channels.xianyu.login.qrcode import XianyuQrcode


class XianyuChannel(Channel):
    """闲鱼：扫码 + Cookie 续期 + 滑块。"""

    def __init__(self) -> None:
        self._browser = XianyuBrowser()

    def browser(self) -> BrowserPlatform:
        return self._browser

    def qrcode(self) -> QrcodeLogin:
        return XianyuQrcode()

    async def renew_cookies(
        self,
        cookies: list[dict[str, Any]],
        *,
        account_id: str,
        punish_url: str | None = None,
        timeout_secs: int = 180,
    ) -> tuple[bool, str, dict[str, Any]]:
        from channels.xianyu.login.cookie_renew import renew_cookies

        return await renew_cookies(
            cookies,
            account_id=account_id,
            punish_url=punish_url,
            platform=self.channel_type or "xianyu",
            timeout_secs=timeout_secs,
        )

"""1688 扫码登录。"""

from __future__ import annotations

import logging
from typing import Any

from channels.ali1688.browser import Ali1688Browser
from channels.core.login.helpers import has_login_cookie, url_looks_logged_in
from channels.core.login.qrcode import QrcodeLogin
from channels.core.login.session import QrSession

logger = logging.getLogger("dingda.sidecar.qr")


class Ali1688Qrcode(QrcodeLogin):
    """1688 扫码；覆写登录探测以处理淘宝 SSO 中间态。"""

    def __init__(self) -> None:
        super().__init__(Ali1688Browser())

    async def _probe_logged_in(
        self,
        session: QrSession,
        page: Any,
        cookies: list[dict[str, Any]],
    ) -> bool:
        if await super()._probe_logged_in(session, page, cookies):
            return True

        config = self.config
        if not config.sso_cookie_name or not config.sso_cookie_domain_keyword:
            return False

        has_site = has_login_cookie(
            cookies,
            name=config.login_cookie_name,
            domain_keyword=config.cookie_domain_keyword,
        )
        if has_site:
            return False

        if not has_login_cookie(
            cookies,
            name=config.sso_cookie_name,
            domain_keyword=config.sso_cookie_domain_keyword,
        ):
            return False

        logger.info(
            "1688 扫码后检测到 SSO Cookie（%s@%s），补访首页完成登录",
            config.sso_cookie_name,
            config.sso_cookie_domain_keyword,
        )
        await self._warm_home(session)
        cookies = await self._read_cookies(session)
        has_site = has_login_cookie(
            cookies,
            name=config.login_cookie_name,
            domain_keyword=config.cookie_domain_keyword,
        )
        return has_site or url_looks_logged_in(
            str(page.url).lower(),
            domain_keyword=config.cookie_domain_keyword,
        )

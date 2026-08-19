"""Auto-generated from contracts/schema."""

from typing import TypedDict

from .channel_cookie import ChannelCookie


class ChannelSidecarCookieRenewRequest(TypedDict, total=False):
    account_id: str
    cookies: list[ChannelCookie]
    punish_url: str
    trace_id: str

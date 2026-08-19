"""Auto-generated from contracts/schema."""

from typing import TypedDict

from .channel_cookie import ChannelCookie


class ChannelSidecarCookieRenewResponse(TypedDict, total=False):
    ok: bool
    status: str
    cookies: list[ChannelCookie]
    detail: str
    trace_id: str

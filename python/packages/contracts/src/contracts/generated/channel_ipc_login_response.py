"""Auto-generated from contracts/schema."""

from typing import TypedDict

from .channel_cookie import ChannelCookie


class ChannelIpcLoginResponse(TypedDict, total=False):
    ok: bool
    state: str
    cookies: list[ChannelCookie]
    detail: str

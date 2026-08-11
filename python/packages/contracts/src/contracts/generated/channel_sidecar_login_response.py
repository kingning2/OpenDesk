"""Auto-generated from contracts/schema."""

from typing import TypedDict

from .channel_cookie import ChannelCookie


class ChannelSidecarLoginResponse(TypedDict, total=False):
    ok: bool
    state: str
    cookies: list[ChannelCookie]
    detail: str
    trace_id: str

"""Auto-generated from contracts/schema."""

from typing import TypedDict
from .channel_cookie import ChannelCookie


class ChannelSidecarLoginProbeRequest(TypedDict, total=False):
    account_id: str
    cookies: list[ChannelCookie]
    headed: bool
    platform: str
    trace_id: str

"""Auto-generated from contracts/schema."""

from typing import TypedDict
from .channel_cookie import ChannelCookie


class ChannelSidecarSearchRequest(TypedDict, total=False):
    account_id: str
    keyword: str
    cookies: list[ChannelCookie]
    max_results: int
    headed: bool
    platform: str
    trace_id: str

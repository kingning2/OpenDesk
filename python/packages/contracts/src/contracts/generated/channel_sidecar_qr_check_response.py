"""Auto-generated from contracts/schema."""

from typing import TypedDict

from .channel_cookie import ChannelCookie


class ChannelSidecarQrCheckResponse(TypedDict, total=False):
    ok: bool
    status: str
    session_id: str
    cookies: list[ChannelCookie]
    detail: str
    qr_base64: str
    trace_id: str

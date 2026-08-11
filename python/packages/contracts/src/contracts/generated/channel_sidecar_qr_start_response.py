"""Auto-generated from contracts/schema."""

from typing import TypedDict


class ChannelSidecarQrStartResponse(TypedDict, total=False):
    ok: bool
    status: str
    session_id: str
    qr_base64: str
    detail: str
    trace_id: str

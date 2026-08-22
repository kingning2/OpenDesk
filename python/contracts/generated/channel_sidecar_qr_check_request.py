"""Auto-generated from contracts/schema."""

from typing import TypedDict


class ChannelSidecarQrCheckRequest(TypedDict, total=False):
    session_id: str
    trace_id: str
    platform: str

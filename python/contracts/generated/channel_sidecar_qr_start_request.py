"""Auto-generated from contracts/schema."""

from typing import TypedDict


class ChannelSidecarQrStartRequest(TypedDict, total=False):
    account_id: str
    trace_id: str
    platform: str

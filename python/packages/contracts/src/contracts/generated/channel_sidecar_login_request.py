"""Auto-generated from contracts/schema."""

from typing import TypedDict


class ChannelSidecarLoginRequest(TypedDict, total=False):
    account_id: str
    credential: str
    trace_id: str

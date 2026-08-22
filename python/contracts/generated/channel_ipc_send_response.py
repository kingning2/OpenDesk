"""Auto-generated from contracts/schema."""

from typing import TypedDict


class ChannelIpcSendResponse(TypedDict, total=False):
    ok: bool
    message_id: str
    detail: str

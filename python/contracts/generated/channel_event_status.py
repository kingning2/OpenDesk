"""Auto-generated from contracts/schema."""

from typing import TypedDict


class ChannelEventStatus(TypedDict, total=False):
    account_id: str
    state: str
    detail: str

"""Auto-generated from contracts/schema."""

from typing import TypedDict

from .channel_message import ChannelMessage


class ChannelEventMessage(TypedDict, total=False):
    account_id: str
    message: ChannelMessage
    suggestion: str

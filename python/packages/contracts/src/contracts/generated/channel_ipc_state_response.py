"""Auto-generated from contracts/schema."""

from typing import TypedDict

from .channel_account import ChannelAccount
from .channel_conversation import ChannelConversation
from .channel_message import ChannelMessage
from .channel_settings import ChannelSettings


class ChannelIpcStateResponse(TypedDict):
    accounts: list[ChannelAccount]
    conversations: list[ChannelConversation]
    messages: list[ChannelMessage]
    settings: ChannelSettings

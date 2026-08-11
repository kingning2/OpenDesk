"""Auto-generated from contracts/schema."""

from typing import TypedDict

from .channel_account import ChannelAccount
from .channel_settings import ChannelSettings


class ChannelIpcStateRequest(TypedDict):
    accounts: list[ChannelAccount]
    settings: ChannelSettings

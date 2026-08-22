"""Auto-generated from contracts/schema."""

from typing import TypedDict


class ChannelAccount(TypedDict):
    id: str
    kind: str
    name: str
    credential: str
    enabled: bool

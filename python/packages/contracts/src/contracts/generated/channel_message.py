"""Auto-generated from contracts/schema."""

from typing import TypedDict


class ChannelMessage(TypedDict):
    id: str
    conversation_id: str
    direction: str
    sender: str
    content: str
    created_at: str

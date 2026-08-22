"""Auto-generated from contracts/schema."""

from typing import TypedDict


class ChannelConversation(TypedDict, total=False):
    id: str
    account_id: str
    cid: str
    peer_id: str
    peer_name: str
    item_id: str
    item_title: str
    item_price: int
    updated_at: str

"""Auto-generated from contracts/schema."""

from typing import TypedDict


class ChannelXianyuSearchItem(TypedDict, total=False):
    itemId: str
    title: str
    url: str
    image: str
    price: str
    location: str
    seller: str
    wantCount: str
    publishedAt: str
    tags: list[str]

"""Auto-generated from contracts/schema."""

from typing import TypedDict


class ChannelAli1688SearchOffer(TypedDict, total=False):
    offerId: str
    title: str
    price: str
    supplier: str
    location: str
    tags: list[str]
    turnover: str
    isP4P: bool
    url: str
    image: str

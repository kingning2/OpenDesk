"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerDtoChannelResult(TypedDict, total=False):
    platform: str
    channel_id: str
    title: str
    country: str
    subscriber_count: int
    email: str
    description: str
    custom_url: str
    email_status: str
    enrich_attempts: int
    enrich_error: str
    enriched_at: str

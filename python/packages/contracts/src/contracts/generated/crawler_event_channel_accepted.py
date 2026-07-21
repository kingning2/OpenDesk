"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerEventChannelAccepted(TypedDict, total=False):
    event_id: str
    occurred_at: str
    job_id: str
    platform: str
    keyword: str
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

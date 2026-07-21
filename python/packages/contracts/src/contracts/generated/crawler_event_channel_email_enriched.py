"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerEventChannelEmailEnriched(TypedDict, total=False):
    event_id: str
    occurred_at: str
    job_id: str
    platform: str
    channel_id: str
    email: str
    email_status: str
    enrich_attempts: int
    enrich_error: str
    enriched_at: str

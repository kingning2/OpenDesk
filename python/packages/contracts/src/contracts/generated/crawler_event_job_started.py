"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerEventJobStarted(TypedDict, total=False):
    event_id: str
    occurred_at: str
    job_id: str
    platform: str
    keywords: str

"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerEventJobCompleted(TypedDict, total=False):
    event_id: str
    occurred_at: str
    job_id: str
    platform: str
    stop_reason: str
    scanned_count: int
    accepted_count: int
    quota_used: int
    duration_ms: int

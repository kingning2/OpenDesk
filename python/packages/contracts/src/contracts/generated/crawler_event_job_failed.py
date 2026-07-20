"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerEventJobFailed(TypedDict):
    event_id: str
    occurred_at: str
    job_id: str
    platform: str
    error_code: str
    message: str

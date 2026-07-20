"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerEventJobLog(TypedDict, total=False):
    event_id: str
    occurred_at: str
    job_id: str
    platform: str
    seq: int
    phase: str
    level: str
    message: str
    keyword: str
    detail: str

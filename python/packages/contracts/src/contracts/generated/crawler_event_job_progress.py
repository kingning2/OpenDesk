"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerEventJobProgress(TypedDict, total=False):
    event_id: str
    occurred_at: str
    job_id: str
    platform: str
    current_keyword: str
    scanned_count: int
    accepted_count: int
    quota_used: int
    search_pages: int
    keyword_scanned: int
    keyword_accepted: int

"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerIpcJobStatusResponse(TypedDict, total=False):
    ok: bool
    job_id: str
    platform: str
    status: str
    stop_reason: str
    message: str
    current_keyword: str
    scanned_count: int
    accepted_count: int
    keyword_scanned: int
    keyword_accepted: int
    quota_used: int
    keywords_total: int
    keywords_done: int
    keyword_stats_json: str
    error_message: str
    trace_id: str

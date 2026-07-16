"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerIpcJobStatusResponse(TypedDict, total=False):
    ok: bool
    job_id: str
    platform: str
    status: str
    stop_reason: str
    scanned_count: int
    accepted_count: int
    trace_id: str

"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerIpcJobStatusRequest(TypedDict, total=False):
    trace_id: str
    job_id: str

"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerIpcJobStartResponse(TypedDict, total=False):
    ok: bool
    job_id: str
    trace_id: str

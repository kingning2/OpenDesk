"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerSidecarJobStartResponse(TypedDict, total=False):
    ok: bool
    job_id: str
    trace_id: str

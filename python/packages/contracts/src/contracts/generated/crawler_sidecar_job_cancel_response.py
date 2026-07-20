"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerSidecarJobCancelResponse(TypedDict, total=False):
    ok: bool
    job_id: str
    trace_id: str

"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerSidecarJobCancelRequest(TypedDict, total=False):
    trace_id: str
    job_id: str

"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerSidecarJobLogsRequest(TypedDict, total=False):
    trace_id: str
    job_id: str

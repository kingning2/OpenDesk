"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerSidecarJobLogsResponse(TypedDict, total=False):
    ok: bool
    job_id: str
    logs_json: str
    trace_id: str

"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerIpcJobResultsResponse(TypedDict, total=False):
    ok: bool
    job_id: str
    results_json: str
    trace_id: str

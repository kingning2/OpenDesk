"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerIpcKeywordsBatchesResponse(TypedDict, total=False):
    ok: bool
    batches_json: str
    trace_id: str

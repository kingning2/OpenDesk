"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerIpcKeywordsImportResponse(TypedDict, total=False):
    ok: bool
    batch_id: str
    inserted: int
    skipped_existing: int
    skipped_too_long: int
    total: int
    trace_id: str
    message: str

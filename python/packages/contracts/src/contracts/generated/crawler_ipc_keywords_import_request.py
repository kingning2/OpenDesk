"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerIpcKeywordsImportRequest(TypedDict, total=False):
    trace_id: str
    csv_content: str
    batch_id: str

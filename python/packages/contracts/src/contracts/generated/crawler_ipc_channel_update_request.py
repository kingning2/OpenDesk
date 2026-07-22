"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerIpcChannelUpdateRequest(TypedDict, total=False):
    trace_id: str
    id: int
    verified_email: str

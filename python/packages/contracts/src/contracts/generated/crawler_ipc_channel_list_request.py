"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerIpcChannelListRequest(TypedDict, total=False):
    trace_id: str
    search: str
    keyword: str
    country: str
    has_email: bool
    email_status: str
    limit: int
    offset: int

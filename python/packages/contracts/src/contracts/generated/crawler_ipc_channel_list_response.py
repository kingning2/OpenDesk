"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerIpcChannelListResponse(TypedDict, total=False):
    ok: bool
    channels_json: str
    total: int
    trace_id: str

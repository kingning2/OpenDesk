"""Auto-generated from contracts/schema."""

from typing import TypedDict


class ChannelSidecarSearchResponse(TypedDict, total=False):
    ok: bool
    status: str
    keyword: str
    total: int
    total_before_filter: int
    offers: list[str]
    final_url: str
    detail: str
    trace_id: str

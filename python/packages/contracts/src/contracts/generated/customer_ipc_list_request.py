"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CustomerIpcListRequest(TypedDict, total=False):
    trace_id: str
    search: str
    limit: int
    offset: int

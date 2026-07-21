"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CustomerIpcListResponse(TypedDict, total=False):
    ok: bool
    customers_json: str
    total: int
    trace_id: str

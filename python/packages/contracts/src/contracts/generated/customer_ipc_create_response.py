"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CustomerIpcCreateResponse(TypedDict, total=False):
    ok: bool
    profile_json: str
    trace_id: str

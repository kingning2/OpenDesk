"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerIpcChannelUpdateResponse(TypedDict, total=False):
    ok: bool
    id: int
    verified_email: str
    email_status: str
    trace_id: str

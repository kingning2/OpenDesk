"""Auto-generated from contracts/schema."""

from typing import TypedDict


class RuntimeEventSidecarRestarted(TypedDict, total=False):
    event_id: str
    occurred_at: str
    port: int
    attempt: int
    reason: str

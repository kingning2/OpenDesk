"""Auto-generated from contracts/schema."""

from typing import TypedDict


class RuntimeEventError(TypedDict, total=False):
    event_id: str
    occurred_at: str
    kind: str
    stage: str
    message: str
    detail: str

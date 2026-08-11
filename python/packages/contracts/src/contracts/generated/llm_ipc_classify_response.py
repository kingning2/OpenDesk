"""Auto-generated from contracts/schema."""

from typing import TypedDict


class LlmIpcClassifyResponse(TypedDict, total=False):
    intent: str
    trace_id: str

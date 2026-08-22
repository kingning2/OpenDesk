"""Auto-generated from contracts/schema."""

from typing import TypedDict


class LlmIpcChatResponse(TypedDict, total=False):
    reply: str
    trace_id: str

"""Auto-generated from contracts/schema."""

from typing import TypedDict

from .llm_message import LlmMessage
from .llm_provider import LlmProvider


class LlmIpcChatRequest(TypedDict, total=False):
    messages: list[LlmMessage]
    provider: LlmProvider
    trace_id: str

"""Auto-generated from contracts/schema."""

from typing import TypedDict
from .llm_provider import LlmProvider


class LlmIpcClassifyRequest(TypedDict, total=False):
    text: str
    scenario: str
    options: list[str]
    provider: LlmProvider
    trace_id: str

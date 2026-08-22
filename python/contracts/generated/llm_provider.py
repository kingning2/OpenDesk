"""Auto-generated from contracts/schema."""

from typing import TypedDict


class LlmProvider(TypedDict):
    base_url: str
    api_key: str
    model: str

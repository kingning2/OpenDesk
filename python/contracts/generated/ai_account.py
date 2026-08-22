"""Auto-generated from contracts/schema."""

from typing import TypedDict


class AiAccount(TypedDict, total=False):
    id: str
    provider_id: str
    name: str
    api_key: str
    default_model: str

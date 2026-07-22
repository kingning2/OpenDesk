"""Auto-generated from contracts/schema."""

from typing import TypedDict


class RuntimeDtoLlmSettings(TypedDict, total=False):
    provider: str
    base_url: str
    model_id: str
    configured: bool
    has_api_key: bool

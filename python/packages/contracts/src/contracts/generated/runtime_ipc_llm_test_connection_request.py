"""Auto-generated from contracts/schema."""

from typing import TypedDict


class RuntimeIpcLlmTestConnectionRequest(TypedDict, total=False):
    provider: str
    base_url: str
    model_id: str
    api_key: str

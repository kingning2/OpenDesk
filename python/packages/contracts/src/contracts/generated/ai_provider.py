"""Auto-generated from contracts/schema."""

from typing import TypedDict


class AiProvider(TypedDict, total=False):
    id: str
    kind: str
    name: str
    base_url: str
    default_model: str

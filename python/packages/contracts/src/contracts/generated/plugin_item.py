"""Auto-generated from contracts/schema."""

from typing import TypedDict


class PluginItem(TypedDict, total=False):
    id: str
    name: str
    description: str
    status: str
    error: str

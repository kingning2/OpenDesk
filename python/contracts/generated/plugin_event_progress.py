"""Auto-generated from contracts/schema."""

from typing import TypedDict


class PluginEventProgress(TypedDict):
    plugin_id: str
    received_bytes: int
    total_bytes: int
    file_name: str

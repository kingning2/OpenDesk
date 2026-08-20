"""Auto-generated from contracts/schema."""

from typing import TypedDict
from .plugin_item import PluginItem


class PluginIpcListResponse(TypedDict):
    items: list[PluginItem]

"""Capabilities declared by a language model implementation."""

from enum import StrEnum


class ModelCapability(StrEnum):
    """Provider-neutral model capabilities used for routing decisions."""

    CHAT = "chat"
    STREAMING = "streaming"
    TOOL_CALLING = "tool_calling"

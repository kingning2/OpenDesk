"""Language model ports implemented by provider adapters."""

from __future__ import annotations

from collections.abc import AsyncIterator
from typing import Protocol, runtime_checkable

from model import ChatRequest, ChatResponse, ChatStreamChunk, ModelCapability


@runtime_checkable
class ChatModel(Protocol):
    """Asynchronous provider-neutral chat model boundary."""

    @property
    def provider_id(self) -> str:
        """Return the stable identifier used by the provider registry."""
        ...

    @property
    def capabilities(self) -> frozenset[ModelCapability]:
        """Return capabilities supported by this implementation."""
        ...

    async def generate(self, request: ChatRequest) -> ChatResponse:
        """Generate one non-streaming assistant response."""
        ...


@runtime_checkable
class StreamingChatModel(ChatModel, Protocol):
    """Chat model that also yields provider-neutral response increments."""

    def stream(self, request: ChatRequest) -> AsyncIterator[ChatStreamChunk]:
        """Stream text deltas and optional completion metadata."""
        ...

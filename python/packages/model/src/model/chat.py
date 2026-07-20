"""Immutable chat request and response values used inside the Python runtime."""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import StrEnum
from math import isfinite


class MessageRole(StrEnum):
    """Roles supported by the minimal chat model boundary."""

    SYSTEM = "system"
    USER = "user"
    ASSISTANT = "assistant"


@dataclass(frozen=True, slots=True)
class ChatMessage:
    """A provider-neutral chat message."""

    role: MessageRole
    content: str
    name: str | None = None

    def __post_init__(self) -> None:
        if not self.content.strip():
            raise ValueError("message content must not be empty")
        if self.name is not None and not self.name.strip():
            raise ValueError("message name must not be empty")


@dataclass(frozen=True, slots=True)
class ChatRequest:
    """Input accepted by every chat model implementation."""

    messages: tuple[ChatMessage, ...]
    model: str | None = None
    temperature: float | None = None
    max_output_tokens: int | None = None

    def __post_init__(self) -> None:
        object.__setattr__(self, "messages", tuple(self.messages))
        if not self.messages:
            raise ValueError("chat request must contain at least one message")
        if self.model is not None and not self.model.strip():
            raise ValueError("model must not be empty")
        if self.temperature is not None and (
            not isfinite(self.temperature) or self.temperature < 0
        ):
            raise ValueError("temperature must be a finite non-negative number")
        if self.max_output_tokens is not None and self.max_output_tokens <= 0:
            raise ValueError("max_output_tokens must be greater than zero")


@dataclass(frozen=True, slots=True)
class TokenUsage:
    """Normalized token usage reported by a provider."""

    input_tokens: int = 0
    output_tokens: int = 0

    def __post_init__(self) -> None:
        if self.input_tokens < 0 or self.output_tokens < 0:
            raise ValueError("token usage must not be negative")

    @property
    def total_tokens(self) -> int:
        return self.input_tokens + self.output_tokens


@dataclass(frozen=True, slots=True)
class ChatStreamChunk:
    """One provider-neutral increment from a streaming chat response."""

    provider: str
    model: str
    text_delta: str = ""
    finish_reason: str | None = None
    usage: TokenUsage | None = None

    def __post_init__(self) -> None:
        if not isinstance(self.provider, str) or not self.provider.strip():
            raise ValueError("provider must not be empty")
        if not isinstance(self.model, str) or not self.model.strip():
            raise ValueError("model must not be empty")
        if not isinstance(self.text_delta, str):
            raise ValueError("text_delta must be a string")
        if self.finish_reason is not None and (
            not isinstance(self.finish_reason, str) or not self.finish_reason.strip()
        ):
            raise ValueError("finish_reason must not be empty")
        if not self.text_delta and self.finish_reason is None and self.usage is None:
            raise ValueError("stream chunk must contain text or metadata")


@dataclass(frozen=True, slots=True)
class ChatResponse:
    """Normalized result returned by a chat model implementation."""

    message: ChatMessage
    provider: str
    model: str
    finish_reason: str = "stop"
    usage: TokenUsage = field(default_factory=TokenUsage)

    def __post_init__(self) -> None:
        if self.message.role is not MessageRole.ASSISTANT:
            raise ValueError("chat response message must use the assistant role")
        if not self.provider.strip():
            raise ValueError("provider must not be empty")
        if not self.model.strip():
            raise ValueError("model must not be empty")
        if not self.finish_reason.strip():
            raise ValueError("finish_reason must not be empty")

"""Deterministic chat model for unit tests and offline development."""

from __future__ import annotations

from llm import ChatModel
from model import (
    ChatMessage,
    ChatRequest,
    ChatResponse,
    MessageRole,
    ModelCapability,
    TokenUsage,
)


class FakeChatModel(ChatModel):
    """Return a configured response without performing external I/O."""

    def __init__(
        self,
        response_text: str = "fake response",
        *,
        provider_id: str = "fake",
        model: str = "fake-model",
    ) -> None:
        if not response_text.strip():
            raise ValueError("response_text must not be empty")
        if not provider_id.strip():
            raise ValueError("provider_id must not be empty")
        if not model.strip():
            raise ValueError("model must not be empty")
        self._response_text = response_text
        self._provider_id = provider_id
        self._model = model
        self.requests: list[ChatRequest] = []

    @property
    def provider_id(self) -> str:
        return self._provider_id

    @property
    def capabilities(self) -> frozenset[ModelCapability]:
        return frozenset({ModelCapability.CHAT})

    async def generate(self, request: ChatRequest) -> ChatResponse:
        self.requests.append(request)
        selected_model = request.model or self._model
        return ChatResponse(
            message=ChatMessage(role=MessageRole.ASSISTANT, content=self._response_text),
            provider=self.provider_id,
            model=selected_model,
            usage=TokenUsage(),
        )

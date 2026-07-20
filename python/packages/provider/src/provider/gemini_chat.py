"""Google Gemini adapter for the provider-neutral chat port."""

from __future__ import annotations

import re
from collections.abc import AsyncIterator, Mapping
from dataclasses import dataclass, field
from math import isfinite

import httpx
from google import genai
from google.genai import errors, types

from llm import (
    InvalidRequestError,
    ProviderAuthenticationError,
    ProviderConfigurationError,
    ProviderErrorMetadata,
    ProviderPermissionDeniedError,
    ProviderRateLimitedError,
    ProviderResponseError,
    ProviderTimeoutError,
    ProviderUnavailableError,
    StreamingChatModel,
)
from model import (
    ChatMessage,
    ChatRequest,
    ChatResponse,
    ChatStreamChunk,
    MessageRole,
    ModelCapability,
    TokenUsage,
)

DEFAULT_GEMINI_MODEL = "gemini-3.5-flash"
_SAFE_PROVIDER_IDENTIFIER = re.compile(r"[A-Za-z0-9][A-Za-z0-9._:/-]{0,127}")


@dataclass(frozen=True, slots=True)
class GeminiChatConfig:
    """Explicit configuration for the Gemini Developer API."""

    api_key: str = field(repr=False)
    model: str = DEFAULT_GEMINI_MODEL
    timeout_seconds: float = 60.0
    max_retries: int = 2

    def __post_init__(self) -> None:
        if not isinstance(self.api_key, str) or not self.api_key.strip():
            raise ProviderConfigurationError("Gemini API key must not be empty")
        if not isinstance(self.model, str) or not self.model.strip():
            raise ProviderConfigurationError("Gemini model must not be empty")
        if (
            isinstance(self.timeout_seconds, bool)
            or not isinstance(self.timeout_seconds, (int, float))
            or not isfinite(self.timeout_seconds)
            or self.timeout_seconds <= 0
        ):
            raise ProviderConfigurationError("Gemini timeout must be a positive number")
        if (
            isinstance(self.max_retries, bool)
            or not isinstance(self.max_retries, int)
            or self.max_retries < 0
        ):
            raise ProviderConfigurationError("Gemini max retries must not be negative")


class GeminiChatModel(StreamingChatModel):
    """Generate complete or streamed text through Google GenAI SDK."""

    def __init__(
        self,
        config: GeminiChatConfig,
        *,
        client: genai.Client | None = None,
    ) -> None:
        self._config = config
        if client is None:
            try:
                client = genai.Client(
                    api_key=config.api_key,
                    http_options=types.HttpOptions(
                        timeout=max(1, int(config.timeout_seconds * 1000)),
                        retry_options=types.HttpRetryOptions(
                            attempts=config.max_retries + 1,
                        ),
                    ),
                )
            except (TypeError, ValueError):
                raise ProviderConfigurationError(
                    "Gemini provider configuration is invalid"
                ) from None
        self._client = client
        self._async_client = client.aio

    @property
    def provider_id(self) -> str:
        return "gemini"

    @property
    def capabilities(self) -> frozenset[ModelCapability]:
        return frozenset({ModelCapability.CHAT, ModelCapability.STREAMING})

    async def generate(self, request: ChatRequest) -> ChatResponse:
        selected_model, contents, config = self._prepare_request(request)

        try:
            response = await self._async_client.models.generate_content(
                model=selected_model,
                contents=contents,
                config=config,
            )
        except errors.APIError as error:
            self._raise_api_error(error)
        except httpx.TimeoutException:
            raise ProviderTimeoutError("Gemini provider request timed out") from None
        except httpx.HTTPError:
            raise ProviderUnavailableError("Gemini provider is unavailable") from None

        return self._normalize_response(response, selected_model)

    async def stream(self, request: ChatRequest) -> AsyncIterator[ChatStreamChunk]:
        selected_model, contents, config = self._prepare_request(request)
        emitted_text = False

        try:
            responses = await self._async_client.models.generate_content_stream(
                model=selected_model,
                contents=contents,
                config=config,
            )
            async for response in responses:
                chunk = self._normalize_stream_chunk(response, selected_model)
                if chunk is None:
                    continue
                emitted_text = emitted_text or bool(chunk.text_delta)
                yield chunk
        except errors.APIError as error:
            self._raise_api_error(error)
        except httpx.TimeoutException:
            raise ProviderTimeoutError("Gemini provider request timed out") from None
        except httpx.HTTPError:
            raise ProviderUnavailableError("Gemini provider is unavailable") from None

        if not emitted_text:
            raise ProviderResponseError("Gemini provider returned no streamed text")

    def _prepare_request(
        self, request: ChatRequest
    ) -> tuple[str, list[types.Content], types.GenerateContentConfig]:
        selected_model = request.model or self._config.model
        contents, system_instructions = self._to_gemini_contents(request.messages)
        config_options: dict[str, object] = {}
        if system_instructions:
            config_options["system_instruction"] = system_instructions
        if request.temperature is not None:
            config_options["temperature"] = request.temperature
        if request.max_output_tokens is not None:
            config_options["max_output_tokens"] = request.max_output_tokens
        return selected_model, contents, types.GenerateContentConfig(**config_options)

    @staticmethod
    def _to_gemini_contents(
        messages: tuple[ChatMessage, ...],
    ) -> tuple[list[types.Content], list[str]]:
        contents: list[types.Content] = []
        system_instructions: list[str] = []
        for message in messages:
            if message.name is not None:
                raise InvalidRequestError("Gemini provider does not support named messages")
            if message.role is MessageRole.SYSTEM:
                system_instructions.append(message.content)
                continue

            role = "model" if message.role is MessageRole.ASSISTANT else "user"
            contents.append(
                types.Content(
                    role=role,
                    parts=[types.Part.from_text(text=message.content)],
                )
            )

        if not contents:
            raise InvalidRequestError("Gemini request must contain user or assistant content")
        return contents, system_instructions

    @classmethod
    def _raise_api_error(cls, error: errors.APIError) -> None:
        status_code = error.code
        metadata = cls._error_metadata(error)
        if status_code == 401:
            raise ProviderAuthenticationError(
                "Gemini provider authentication failed",
                metadata=metadata,
            ) from None
        if status_code == 403:
            raise ProviderPermissionDeniedError(
                "Gemini provider permission denied",
                metadata=metadata,
            ) from None
        if status_code == 408:
            raise ProviderTimeoutError(
                "Gemini provider request timed out",
                metadata=metadata,
            ) from None
        if status_code == 429:
            raise ProviderRateLimitedError(
                "Gemini provider rate limit exceeded",
                metadata=metadata,
            ) from None
        if 400 <= status_code < 500:
            raise InvalidRequestError(
                "Gemini provider rejected the request",
                metadata=metadata,
            ) from None
        if status_code >= 500:
            raise ProviderUnavailableError(
                "Gemini provider is unavailable",
                metadata=metadata,
            ) from None
        raise ProviderResponseError(
            "Gemini provider request failed",
            metadata=metadata,
        ) from None

    @classmethod
    def _error_metadata(cls, error: errors.APIError) -> ProviderErrorMetadata:
        request_id: object | None = None
        response = error.response
        headers = getattr(response, "headers", None)
        if isinstance(headers, Mapping):
            request_id = headers.get("x-request-id") or headers.get("x-goog-request-id")

        return ProviderErrorMetadata(
            status_code=error.code if 100 <= error.code <= 599 else None,
            error_code=cls._safe_identifier(error.status),
            request_id=cls._safe_identifier(request_id),
        )

    @staticmethod
    def _safe_identifier(value: object | None) -> str | None:
        if isinstance(value, str) and _SAFE_PROVIDER_IDENTIFIER.fullmatch(value):
            return value
        return None

    def _normalize_response(self, response: object, selected_model: str) -> ChatResponse:
        try:
            candidates = response.candidates  # type: ignore[attr-defined]
            if not candidates:
                raise ProviderResponseError("Gemini provider returned no candidates")

            content = response.text  # type: ignore[attr-defined]
            if not isinstance(content, str) or not content.strip():
                raise ProviderResponseError("Gemini provider returned no text content")

            response_model = getattr(response, "model_version", None)
            if not isinstance(response_model, str) or not response_model.strip():
                response_model = selected_model

            finish_reason = self._normalize_finish_reason(
                getattr(candidates[0], "finish_reason", None)
            )
            usage = self._normalize_usage(getattr(response, "usage_metadata", None))
        except ProviderResponseError:
            raise
        except (AttributeError, IndexError, TypeError, ValueError):
            raise ProviderResponseError("Gemini provider returned an invalid response") from None

        return ChatResponse(
            message=ChatMessage(role=MessageRole.ASSISTANT, content=content),
            provider=self.provider_id,
            model=response_model,
            finish_reason=finish_reason,
            usage=usage,
        )

    def _normalize_stream_chunk(
        self, response: object, selected_model: str
    ) -> ChatStreamChunk | None:
        try:
            content = getattr(response, "text", None)
            if content is None:
                content = ""
            if not isinstance(content, str):
                raise ProviderResponseError("Gemini provider returned invalid streamed text")

            response_model = getattr(response, "model_version", None)
            if not isinstance(response_model, str) or not response_model.strip():
                response_model = selected_model

            candidates = getattr(response, "candidates", None) or []
            finish_reason = None
            if candidates:
                finish_reason = self._normalize_optional_finish_reason(
                    getattr(candidates[0], "finish_reason", None)
                )

            raw_usage = getattr(response, "usage_metadata", None)
            usage = self._normalize_usage(raw_usage) if raw_usage is not None else None
        except ProviderResponseError:
            raise
        except (AttributeError, IndexError, TypeError, ValueError):
            raise ProviderResponseError(
                "Gemini provider returned an invalid streamed response"
            ) from None

        if not content and finish_reason is None and usage is None:
            return None
        return ChatStreamChunk(
            provider=self.provider_id,
            model=response_model,
            text_delta=content,
            finish_reason=finish_reason,
            usage=usage,
        )

    @staticmethod
    def _normalize_finish_reason(value: object | None) -> str:
        return GeminiChatModel._normalize_optional_finish_reason(value) or "unknown"

    @staticmethod
    def _normalize_optional_finish_reason(value: object | None) -> str | None:
        raw_value = getattr(value, "value", value)
        if not isinstance(raw_value, str) or not raw_value.strip():
            return None
        return raw_value.lower()

    @classmethod
    def _normalize_usage(cls, usage: object | None) -> TokenUsage:
        if usage is None:
            return TokenUsage()

        input_tokens = cls._token_count(getattr(usage, "prompt_token_count", None))
        total_tokens = getattr(usage, "total_token_count", None)
        if total_tokens is not None:
            normalized_total = cls._token_count(total_tokens)
            if normalized_total < input_tokens:
                raise ProviderResponseError("Gemini provider returned invalid token usage")
            output_tokens = normalized_total - input_tokens
        else:
            output_tokens = cls._token_count(
                getattr(usage, "candidates_token_count", None)
            ) + cls._token_count(getattr(usage, "thoughts_token_count", None))

        return TokenUsage(input_tokens=input_tokens, output_tokens=output_tokens)

    @staticmethod
    def _token_count(value: object | None) -> int:
        if value is None:
            return 0
        if not isinstance(value, int) or isinstance(value, bool) or value < 0:
            raise ProviderResponseError("Gemini provider returned invalid token usage")
        return value

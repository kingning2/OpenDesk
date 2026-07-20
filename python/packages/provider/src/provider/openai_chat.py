"""OpenAI Chat Completions adapter for the provider-neutral chat port."""

from __future__ import annotations

import re
from collections.abc import Mapping
from dataclasses import dataclass, field
from math import isfinite

from openai import (
    APIConnectionError,
    APIResponseValidationError,
    APIStatusError,
    APITimeoutError,
    AsyncOpenAI,
    AuthenticationError,
    BadRequestError,
    OpenAIError,
    PermissionDeniedError,
    RateLimitError,
)

from llm import (
    ChatModel,
    InvalidRequestError,
    ProviderAuthenticationError,
    ProviderConfigurationError,
    ProviderErrorMetadata,
    ProviderPermissionDeniedError,
    ProviderRateLimitedError,
    ProviderResponseError,
    ProviderTimeoutError,
    ProviderUnavailableError,
)
from model import (
    ChatMessage,
    ChatRequest,
    ChatResponse,
    MessageRole,
    ModelCapability,
    TokenUsage,
)

_SAFE_PROVIDER_IDENTIFIER = re.compile(r"[A-Za-z0-9][A-Za-z0-9._:/-]{0,127}")


@dataclass(frozen=True, slots=True)
class OpenAIChatConfig:
    """Explicit configuration for an OpenAI-compatible chat endpoint."""

    model: str
    api_key: str = field(repr=False)
    base_url: str | None = None
    timeout_seconds: float = 60.0
    max_retries: int = 2

    def __post_init__(self) -> None:
        if not isinstance(self.model, str) or not self.model.strip():
            raise ProviderConfigurationError("OpenAI model must not be empty")
        if not isinstance(self.api_key, str) or not self.api_key.strip():
            raise ProviderConfigurationError("OpenAI API key must not be empty")
        if self.base_url is not None and (
            not isinstance(self.base_url, str) or not self.base_url.strip()
        ):
            raise ProviderConfigurationError("OpenAI base URL must not be empty")
        if (
            isinstance(self.timeout_seconds, bool)
            or not isinstance(self.timeout_seconds, (int, float))
            or not isfinite(self.timeout_seconds)
            or self.timeout_seconds <= 0
        ):
            raise ProviderConfigurationError("OpenAI timeout must be a positive number")
        if (
            isinstance(self.max_retries, bool)
            or not isinstance(self.max_retries, int)
            or self.max_retries < 0
        ):
            raise ProviderConfigurationError("OpenAI max retries must not be negative")


class OpenAIChatModel(ChatModel):
    """Generate one non-streaming chat response through the official OpenAI SDK."""

    def __init__(
        self,
        config: OpenAIChatConfig,
        *,
        client: AsyncOpenAI | None = None,
    ) -> None:
        self._config = config
        if client is not None:
            self._client = client
            return

        try:
            self._client = AsyncOpenAI(
                api_key=config.api_key,
                base_url=config.base_url,
                timeout=config.timeout_seconds,
                max_retries=config.max_retries,
            )
        except (OpenAIError, TypeError, ValueError):
            raise ProviderConfigurationError("OpenAI provider configuration is invalid") from None

    @property
    def provider_id(self) -> str:
        return "openai"

    @property
    def capabilities(self) -> frozenset[ModelCapability]:
        return frozenset({ModelCapability.CHAT})

    async def generate(self, request: ChatRequest) -> ChatResponse:
        selected_model = request.model or self._config.model
        request_options: dict[str, object] = {
            "messages": [self._to_openai_message(message) for message in request.messages],
            "model": selected_model,
            "stream": False,
        }
        if request.temperature is not None:
            request_options["temperature"] = request.temperature
        if request.max_output_tokens is not None:
            request_options["max_completion_tokens"] = request.max_output_tokens

        try:
            response = await self._client.chat.completions.create(**request_options)
        except AuthenticationError as error:
            raise ProviderAuthenticationError(
                "OpenAI provider authentication failed",
                metadata=self._status_metadata(error),
            ) from None
        except PermissionDeniedError as error:
            raise ProviderPermissionDeniedError(
                "OpenAI provider permission denied",
                metadata=self._status_metadata(error),
            ) from None
        except RateLimitError as error:
            raise ProviderRateLimitedError(
                "OpenAI provider rate limit exceeded",
                metadata=self._status_metadata(error),
            ) from None
        except APITimeoutError:
            raise ProviderTimeoutError("OpenAI provider request timed out") from None
        except APIConnectionError:
            raise ProviderUnavailableError("OpenAI provider is unavailable") from None
        except BadRequestError as error:
            raise InvalidRequestError(
                "OpenAI provider rejected the request",
                metadata=self._status_metadata(error),
            ) from None
        except APIStatusError as error:
            self._raise_status_error(error)
        except APIResponseValidationError:
            raise ProviderResponseError("OpenAI provider returned an invalid response") from None
        except OpenAIError:
            raise ProviderResponseError("OpenAI provider request failed") from None

        return self._normalize_response(response, selected_model)

    @staticmethod
    def _to_openai_message(message: ChatMessage) -> dict[str, str]:
        value = {"role": message.role.value, "content": message.content}
        if message.name is not None:
            value["name"] = message.name
        return value

    @classmethod
    def _raise_status_error(cls, error: APIStatusError) -> None:
        status_code = error.status_code
        metadata = cls._status_metadata(error)
        if status_code == 401:
            raise ProviderAuthenticationError(
                "OpenAI provider authentication failed",
                metadata=metadata,
            ) from None
        if status_code == 403:
            raise ProviderPermissionDeniedError(
                "OpenAI provider permission denied",
                metadata=metadata,
            ) from None
        if status_code == 429:
            raise ProviderRateLimitedError(
                "OpenAI provider rate limit exceeded",
                metadata=metadata,
            ) from None
        if status_code == 408:
            raise ProviderTimeoutError(
                "OpenAI provider request timed out",
                metadata=metadata,
            ) from None
        if 400 <= status_code < 500:
            raise InvalidRequestError(
                "OpenAI provider rejected the request",
                metadata=metadata,
            ) from None
        if status_code >= 500:
            raise ProviderUnavailableError(
                "OpenAI provider is unavailable",
                metadata=metadata,
            ) from None
        raise ProviderResponseError(
            "OpenAI provider request failed",
            metadata=metadata,
        ) from None

    @staticmethod
    def _status_metadata(error: APIStatusError) -> ProviderErrorMetadata:
        provider_error_code: object | None = None
        body = error.body
        if isinstance(body, Mapping):
            details = body.get("error", body)
            if isinstance(details, Mapping):
                provider_error_code = details.get("code")

        return ProviderErrorMetadata(
            status_code=error.status_code,
            error_code=OpenAIChatModel._safe_identifier(provider_error_code),
            request_id=OpenAIChatModel._safe_identifier(error.request_id),
        )

    @staticmethod
    def _safe_identifier(value: object | None) -> str | None:
        if isinstance(value, str) and _SAFE_PROVIDER_IDENTIFIER.fullmatch(value):
            return value
        return None

    def _normalize_response(self, response: object, selected_model: str) -> ChatResponse:
        try:
            choices = response.choices  # type: ignore[attr-defined]
            if not choices:
                raise ProviderResponseError("OpenAI provider returned no choices")

            choice = choices[0]
            content = choice.message.content
            if not isinstance(content, str) or not content.strip():
                raise ProviderResponseError("OpenAI provider returned no text content")

            response_model = getattr(response, "model", None)
            if not isinstance(response_model, str) or not response_model.strip():
                response_model = selected_model

            finish_reason = getattr(choice, "finish_reason", None)
            if not isinstance(finish_reason, str) or not finish_reason.strip():
                finish_reason = "unknown"

            usage = self._normalize_usage(getattr(response, "usage", None))
        except ProviderResponseError:
            raise
        except (AttributeError, IndexError, TypeError, ValueError):
            raise ProviderResponseError("OpenAI provider returned an invalid response") from None

        return ChatResponse(
            message=ChatMessage(role=MessageRole.ASSISTANT, content=content),
            provider=self.provider_id,
            model=response_model,
            finish_reason=finish_reason,
            usage=usage,
        )

    @staticmethod
    def _normalize_usage(usage: object | None) -> TokenUsage:
        if usage is None:
            return TokenUsage()

        input_tokens = getattr(usage, "prompt_tokens", 0) or 0
        output_tokens = getattr(usage, "completion_tokens", 0) or 0
        if (
            not isinstance(input_tokens, int)
            or isinstance(input_tokens, bool)
            or input_tokens < 0
            or not isinstance(output_tokens, int)
            or isinstance(output_tokens, bool)
            or output_tokens < 0
        ):
            raise ProviderResponseError("OpenAI provider returned invalid token usage")
        return TokenUsage(input_tokens=input_tokens, output_tokens=output_tokens)

"""Normalized errors exposed by language model ports."""

import re
from dataclasses import dataclass

_SAFE_PROVIDER_IDENTIFIER = re.compile(r"[A-Za-z0-9][A-Za-z0-9._:/-]{0,127}")


@dataclass(frozen=True, slots=True)
class ProviderErrorMetadata:
    """Non-sensitive provider diagnostics safe to expose across the LLM boundary."""

    status_code: int | None = None
    error_code: str | None = None
    request_id: str | None = None

    def __post_init__(self) -> None:
        if self.status_code is not None and (
            isinstance(self.status_code, bool)
            or not isinstance(self.status_code, int)
            or not 100 <= self.status_code <= 599
        ):
            raise ValueError("status_code must be a valid HTTP status")
        self._validate_identifier("error_code", self.error_code)
        self._validate_identifier("request_id", self.request_id)

    @staticmethod
    def _validate_identifier(name: str, value: str | None) -> None:
        if value is not None and not _SAFE_PROVIDER_IDENTIFIER.fullmatch(value):
            raise ValueError(f"{name} must be a safe provider identifier")


class LLMError(Exception):
    """Base error raised through the provider-neutral LLM boundary."""

    code = "llm_error"

    def __init__(
        self,
        message: str = "",
        *,
        metadata: ProviderErrorMetadata | None = None,
    ) -> None:
        super().__init__(message)
        self.metadata = metadata or ProviderErrorMetadata()

    def __str__(self) -> str:
        message = super().__str__()
        fields: list[str] = []
        if self.metadata.status_code is not None:
            fields.append(f"status_code={self.metadata.status_code}")
        if self.metadata.error_code is not None:
            fields.append(f"provider_error_code={self.metadata.error_code}")
        if self.metadata.request_id is not None:
            fields.append(f"request_id={self.metadata.request_id}")
        if not fields:
            return message
        suffix = ", ".join(fields)
        return f"{message} [{suffix}]" if message else f"[{suffix}]"


class InvalidRequestError(LLMError):
    code = "invalid_request"


class ProviderConfigurationError(LLMError):
    code = "provider_configuration_error"


class ProviderAuthenticationError(LLMError):
    code = "provider_authentication_error"


class ProviderPermissionDeniedError(LLMError):
    code = "provider_permission_denied"


class ProviderResponseError(LLMError):
    code = "provider_response_error"


class ProviderUnavailableError(LLMError):
    code = "provider_unavailable"


class ProviderRateLimitedError(LLMError):
    code = "provider_rate_limited"


class ProviderTimeoutError(LLMError):
    code = "provider_timeout"

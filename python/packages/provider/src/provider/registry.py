"""Registry for provider-neutral chat model implementations."""

from __future__ import annotations

from llm import ChatModel


class ProviderRegistryError(Exception):
    """Base provider registry error."""


class DuplicateProviderError(ProviderRegistryError):
    """Raised when a provider identifier is registered twice."""


class UnknownProviderError(ProviderRegistryError):
    """Raised when a provider identifier cannot be resolved."""


class ProviderRegistry:
    """Resolve injected chat models by a stable provider identifier."""

    def __init__(self) -> None:
        self._providers: dict[str, ChatModel] = {}

    @property
    def provider_ids(self) -> tuple[str, ...]:
        return tuple(sorted(self._providers))

    def register(self, provider: ChatModel) -> None:
        provider_id = provider.provider_id.strip()
        if not provider_id:
            raise ValueError("provider_id must not be empty")
        if provider_id in self._providers:
            raise DuplicateProviderError(f"provider already registered: {provider_id}")
        self._providers[provider_id] = provider

    def resolve(self, provider_id: str) -> ChatModel:
        normalized_id = provider_id.strip()
        try:
            return self._providers[normalized_id]
        except KeyError as error:
            raise UnknownProviderError(f"provider is not registered: {normalized_id}") from error

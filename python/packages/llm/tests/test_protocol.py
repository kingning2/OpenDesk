"""Tests for the provider-neutral language model port."""

import unittest

from llm import (
    ChatModel,
    ProviderAuthenticationError,
    ProviderConfigurationError,
    ProviderErrorMetadata,
    ProviderPermissionDeniedError,
    ProviderResponseError,
    ProviderTimeoutError,
    StreamingChatModel,
)
from model import ChatRequest, ChatResponse, ChatStreamChunk, ModelCapability


class StubChatModel:
    @property
    def provider_id(self) -> str:
        return "stub"

    @property
    def capabilities(self) -> frozenset[ModelCapability]:
        return frozenset({ModelCapability.CHAT})

    async def generate(self, request: ChatRequest) -> ChatResponse:
        raise NotImplementedError(request)


class StubStreamingChatModel(StubChatModel):
    async def stream(self, request: ChatRequest):
        del request
        yield ChatStreamChunk(
            provider="stub",
            model="stub-model",
            text_delta="hello",
        )


class ChatModelProtocolTests(unittest.TestCase):
    def test_structural_implementation_satisfies_protocol(self) -> None:
        self.assertIsInstance(StubChatModel(), ChatModel)

    def test_streaming_implementation_satisfies_both_protocols(self) -> None:
        model = StubStreamingChatModel()

        self.assertIsInstance(model, ChatModel)
        self.assertIsInstance(model, StreamingChatModel)

    def test_errors_expose_stable_code(self) -> None:
        self.assertEqual(ProviderTimeoutError.code, "provider_timeout")
        self.assertEqual(
            ProviderConfigurationError.code,
            "provider_configuration_error",
        )
        self.assertEqual(
            ProviderAuthenticationError.code,
            "provider_authentication_error",
        )
        self.assertEqual(
            ProviderPermissionDeniedError.code,
            "provider_permission_denied",
        )
        self.assertEqual(ProviderResponseError.code, "provider_response_error")

    def test_errors_render_only_safe_provider_metadata(self) -> None:
        metadata = ProviderErrorMetadata(
            status_code=403,
            error_code="model_not_found",
            request_id="req_safe123",
        )

        error = ProviderPermissionDeniedError(
            "provider permission denied",
            metadata=metadata,
        )

        self.assertEqual(error.metadata, metadata)
        self.assertEqual(
            str(error),
            "provider permission denied "
            "[status_code=403, provider_error_code=model_not_found, request_id=req_safe123]",
        )

    def test_metadata_rejects_message_like_values(self) -> None:
        with self.assertRaises(ValueError):
            ProviderErrorMetadata(error_code="raw provider error body")


if __name__ == "__main__":
    unittest.main()

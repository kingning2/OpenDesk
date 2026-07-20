"""Tests for provider registry and deterministic fake model."""

import unittest

from llm import ChatModel
from model import ChatMessage, ChatRequest, MessageRole, ModelCapability
from provider import (
    DuplicateProviderError,
    FakeChatModel,
    ProviderRegistry,
    UnknownProviderError,
)


class ProviderRegistryTests(unittest.TestCase):
    def test_registers_and_resolves_provider(self) -> None:
        registry = ProviderRegistry()
        fake = FakeChatModel()

        registry.register(fake)

        self.assertIs(registry.resolve("fake"), fake)
        self.assertEqual(registry.provider_ids, ("fake",))

    def test_rejects_duplicate_provider(self) -> None:
        registry = ProviderRegistry()
        registry.register(FakeChatModel())

        with self.assertRaises(DuplicateProviderError):
            registry.register(FakeChatModel())

    def test_rejects_unknown_provider(self) -> None:
        with self.assertRaises(UnknownProviderError):
            ProviderRegistry().resolve("missing")


class FakeChatModelTests(unittest.IsolatedAsyncioTestCase):
    async def test_returns_deterministic_response_and_records_request(self) -> None:
        fake = FakeChatModel(response_text="hello from fake")
        request = ChatRequest(
            messages=(ChatMessage(role=MessageRole.USER, content="hello"),),
        )

        response = await fake.generate(request)

        self.assertIsInstance(fake, ChatModel)
        self.assertEqual(response.message.content, "hello from fake")
        self.assertEqual(response.provider, "fake")
        self.assertEqual(fake.requests, [request])
        self.assertEqual(fake.capabilities, frozenset({ModelCapability.CHAT}))


if __name__ == "__main__":
    unittest.main()

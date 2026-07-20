"""Tests for provider-neutral chat model values."""

import unittest

from model import (
    ChatMessage,
    ChatRequest,
    ChatResponse,
    ChatStreamChunk,
    MessageRole,
    TokenUsage,
)


class ChatModelValueTests(unittest.TestCase):
    def test_request_normalizes_messages_to_tuple(self) -> None:
        message = ChatMessage(role=MessageRole.USER, content="hello")

        request = ChatRequest(messages=[message])  # type: ignore[arg-type]

        self.assertEqual(request.messages, (message,))

    def test_request_rejects_empty_messages(self) -> None:
        with self.assertRaisesRegex(ValueError, "at least one message"):
            ChatRequest(messages=())

    def test_message_rejects_blank_content(self) -> None:
        with self.assertRaisesRegex(ValueError, "must not be empty"):
            ChatMessage(role=MessageRole.USER, content="  ")

    def test_usage_calculates_total(self) -> None:
        usage = TokenUsage(input_tokens=3, output_tokens=5)

        self.assertEqual(usage.total_tokens, 8)

    def test_response_requires_assistant_message(self) -> None:
        message = ChatMessage(role=MessageRole.USER, content="hello")

        with self.assertRaisesRegex(ValueError, "assistant role"):
            ChatResponse(message=message, provider="fake", model="fake-model")

    def test_stream_chunk_accepts_text_and_final_metadata(self) -> None:
        text_chunk = ChatStreamChunk(
            provider="fake",
            model="fake-model",
            text_delta="hello",
        )
        final_chunk = ChatStreamChunk(
            provider="fake",
            model="fake-model",
            finish_reason="stop",
            usage=TokenUsage(input_tokens=2, output_tokens=1),
        )

        self.assertEqual(text_chunk.text_delta, "hello")
        self.assertEqual(final_chunk.finish_reason, "stop")
        self.assertEqual(final_chunk.usage.total_tokens, 3)

    def test_stream_chunk_rejects_empty_chunk(self) -> None:
        with self.assertRaisesRegex(ValueError, "text or metadata"):
            ChatStreamChunk(provider="fake", model="fake-model")


if __name__ == "__main__":
    unittest.main()

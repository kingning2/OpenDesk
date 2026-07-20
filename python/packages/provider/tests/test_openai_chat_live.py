"""Explicitly enabled live test for an OpenAI-compatible chat endpoint."""

from __future__ import annotations

import os
import unittest

from openai import AsyncOpenAI

from model import ChatMessage, ChatRequest, MessageRole
from provider import OpenAIChatConfig, OpenAIChatModel


@unittest.skipUnless(
    os.environ.get("OPENDESK_RUN_LIVE_LLM_TESTS") == "1"
    and os.environ.get("OPENDESK_LLM_PROVIDER", "").strip().lower() == "openai",
    "set live LLM tests on with OPENDESK_LLM_PROVIDER=openai",
)
class OpenAIChatModelLiveTests(unittest.IsolatedAsyncioTestCase):
    async def test_generates_real_response(self) -> None:
        api_key = os.environ.get("OPENAI_API_KEY")
        model_name = os.environ.get("OPENDESK_LLM_MODEL")
        if not api_key or not model_name:
            self.fail("OPENAI_API_KEY and OPENDESK_LLM_MODEL are required for live tests")

        base_url = os.environ.get("OPENAI_BASE_URL") or None
        config = OpenAIChatConfig(
            model=model_name,
            api_key=api_key,
            base_url=base_url,
        )
        client = AsyncOpenAI(api_key=api_key, base_url=base_url)

        try:
            response = await OpenAIChatModel(config, client=client).generate(
                ChatRequest(
                    messages=(
                        ChatMessage(
                            role=MessageRole.USER,
                            content="Reply with the single word pong.",
                        ),
                    ),
                    max_output_tokens=16,
                )
            )
        finally:
            await client.close()

        self.assertTrue(response.message.content.strip())
        self.assertEqual(response.provider, "openai")


if __name__ == "__main__":
    unittest.main()

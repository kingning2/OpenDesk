"""Explicitly enabled live test for the native Gemini Developer API."""

from __future__ import annotations

import os
import unittest

from google import genai

from model import ChatMessage, ChatRequest, MessageRole
from provider import GeminiChatConfig, GeminiChatModel


@unittest.skipUnless(
    os.environ.get("OPENDESK_RUN_LIVE_LLM_TESTS") == "1"
    and os.environ.get("OPENDESK_LLM_PROVIDER", "").strip().lower() == "gemini",
    "set live LLM tests on with OPENDESK_LLM_PROVIDER=gemini",
)
class GeminiChatModelLiveTests(unittest.IsolatedAsyncioTestCase):
    async def test_streams_real_response(self) -> None:
        api_key = os.environ.get("GEMINI_API_KEY")
        model_name = os.environ.get("OPENDESK_LLM_MODEL") or "gemini-3.5-flash"
        if not api_key:
            self.fail("GEMINI_API_KEY is required for Gemini live tests")

        config = GeminiChatConfig(api_key=api_key, model=model_name)
        client = genai.Client(api_key=api_key)
        async_client = client.aio
        adapter = GeminiChatModel(config, client=client)
        text_parts: list[str] = []
        finish_reason: str | None = None
        usage = None
        response_model = model_name
        chunk_count = 0

        print("\n--- Gemini 流式正文 ---", flush=True)
        try:
            async for chunk in adapter.stream(
                ChatRequest(
                    messages=(
                        ChatMessage(
                            role=MessageRole.USER,
                            content="请用中文分三点简短介绍流式输出，每点一行。",
                        ),
                    ),
                )
            ):
                chunk_count += 1
                response_model = chunk.model
                if chunk.text_delta:
                    text_parts.append(chunk.text_delta)
                    print(chunk.text_delta, end="", flush=True)
                if chunk.finish_reason is not None:
                    finish_reason = chunk.finish_reason
                if chunk.usage is not None:
                    usage = chunk.usage
        finally:
            await async_client.aclose()
            client.close()

        full_text = "".join(text_parts)
        print("\n\n--- Gemini 流式元数据 ---", flush=True)
        print("provider=gemini", flush=True)
        print(f"model={response_model}", flush=True)
        print(f"chunks={chunk_count}", flush=True)
        print(f"finish_reason={finish_reason or 'unknown'}", flush=True)
        if usage is not None:
            print(
                "usage="
                f"input:{usage.input_tokens}, "
                f"output:{usage.output_tokens}, "
                f"total:{usage.total_tokens}",
                flush=True,
            )

        self.assertTrue(full_text.strip())
        self.assertGreater(chunk_count, 0)


if __name__ == "__main__":
    unittest.main()

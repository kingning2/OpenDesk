"""Offline tests for the native Google Gemini adapter."""

from __future__ import annotations

import traceback
import unittest
from types import SimpleNamespace
from unittest.mock import AsyncMock, patch

import httpx
from google.genai import errors, types

from llm import (
    ChatModel,
    InvalidRequestError,
    ProviderAuthenticationError,
    ProviderConfigurationError,
    ProviderPermissionDeniedError,
    ProviderRateLimitedError,
    ProviderResponseError,
    ProviderTimeoutError,
    ProviderUnavailableError,
    StreamingChatModel,
)
from model import ChatMessage, ChatRequest, MessageRole, ModelCapability
from provider import (
    DEFAULT_GEMINI_MODEL,
    GeminiChatConfig,
    GeminiChatModel,
    ProviderRegistry,
)


def make_response(
    *,
    content: str = "hello from Gemini",
    model: str = "gemini-test-response",
    finish_reason: types.FinishReason | None = types.FinishReason.STOP,
    prompt_tokens: int = 10,
    candidates_tokens: int = 5,
    thoughts_tokens: int = 2,
    total_tokens: int | None = 17,
) -> types.GenerateContentResponse:
    return types.GenerateContentResponse(
        candidates=[
            types.Candidate(
                content=types.Content(
                    role="model",
                    parts=[types.Part.from_text(text=content)],
                ),
                finish_reason=finish_reason,
            )
        ],
        model_version=model,
        usage_metadata=types.GenerateContentResponseUsageMetadata(
            prompt_token_count=prompt_tokens,
            candidates_token_count=candidates_tokens,
            thoughts_token_count=thoughts_tokens,
            total_token_count=total_tokens,
        ),
    )


def make_client(
    *, response: object | None = None, error: Exception | None = None
) -> tuple[SimpleNamespace, AsyncMock]:
    generate_content = (
        AsyncMock(side_effect=error) if error is not None else AsyncMock(return_value=response)
    )
    async_client = SimpleNamespace(models=SimpleNamespace(generate_content=generate_content))
    return SimpleNamespace(aio=async_client), generate_content


def make_stream_client(
    *, responses: tuple[object, ...] = (), error: Exception | None = None
) -> tuple[SimpleNamespace, AsyncMock]:
    async def response_stream():
        for response in responses:
            yield response

    generate_content_stream = (
        AsyncMock(side_effect=error)
        if error is not None
        else AsyncMock(return_value=response_stream())
    )
    async_client = SimpleNamespace(
        models=SimpleNamespace(
            generate_content=AsyncMock(),
            generate_content_stream=generate_content_stream,
        )
    )
    return SimpleNamespace(aio=async_client), generate_content_stream


def make_api_error(
    error_type: type[errors.APIError],
    status_code: int,
    status: str,
    message: str,
    *,
    request_id: str = "req_gemini123",
) -> errors.APIError:
    request = httpx.Request("POST", "https://generativelanguage.googleapis.com")
    response = httpx.Response(
        status_code,
        request=request,
        headers={"x-request-id": request_id},
    )
    return error_type(
        status_code,
        {"error": {"code": status_code, "message": message, "status": status}},
        response,
    )


class GeminiChatConfigTests(unittest.TestCase):
    def test_defaults_and_secret_repr(self) -> None:
        api_key_value = "gemini-test-key"
        config = GeminiChatConfig(api_key=api_key_value)

        self.assertEqual(config.model, "gemini-3.5-flash")
        self.assertEqual(config.model, DEFAULT_GEMINI_MODEL)
        self.assertEqual(config.timeout_seconds, 60.0)
        self.assertEqual(config.max_retries, 2)
        self.assertNotIn(api_key_value, repr(config))
        self.assertNotIn("api_key", repr(config))

    def test_rejects_invalid_configuration_without_echoing_secret(self) -> None:
        cases = (
            {"api_key": ""},
            {"api_key": "gemini-test-key", "model": ""},
            {"api_key": "gemini-test-key", "timeout_seconds": 0},
            {"api_key": "gemini-test-key", "max_retries": -1},
        )

        for options in cases:
            with self.subTest(options=options):
                with self.assertRaises(ProviderConfigurationError) as raised:
                    GeminiChatConfig(**options)
                self.assertNotIn("gemini-test-key", str(raised.exception))

    def test_builds_official_client_with_timeout_and_retry_options(self) -> None:
        api_key_value = "gemini-test-key"
        config = GeminiChatConfig(
            api_key=api_key_value,
            model="gemini-test",
            timeout_seconds=12.5,
            max_retries=4,
        )
        injected_client = SimpleNamespace(aio=SimpleNamespace())

        with patch(
            "provider.gemini_chat.genai.Client", return_value=injected_client
        ) as constructor:
            GeminiChatModel(config)

        options = constructor.call_args.kwargs["http_options"]
        self.assertEqual(constructor.call_args.kwargs["api_key"], "gemini-test-key")
        self.assertEqual(options.timeout, 12_500)
        self.assertEqual(options.retry_options.attempts, 5)


class GeminiChatModelTests(unittest.IsolatedAsyncioTestCase):
    def make_model(
        self, *, response: object | None = None, error: Exception | None = None
    ) -> tuple[GeminiChatModel, AsyncMock]:
        client, generate_content = make_client(response=response, error=error)
        api_key_value = "gemini-test-key"
        config = GeminiChatConfig(api_key=api_key_value)
        return GeminiChatModel(config, client=client), generate_content

    def make_stream_model(
        self, *, responses: tuple[object, ...] = (), error: Exception | None = None
    ) -> tuple[GeminiChatModel, AsyncMock]:
        client, generate_content_stream = make_stream_client(
            responses=responses,
            error=error,
        )
        api_key_value = "gemini-test-key"
        config = GeminiChatConfig(api_key=api_key_value)
        return GeminiChatModel(config, client=client), generate_content_stream

    async def test_maps_system_user_and_assistant_messages(self) -> None:
        model, generate_content = self.make_model(response=make_response())
        request = ChatRequest(
            messages=(
                ChatMessage(role=MessageRole.SYSTEM, content="system one"),
                ChatMessage(role=MessageRole.SYSTEM, content="system two"),
                ChatMessage(role=MessageRole.USER, content="user"),
                ChatMessage(role=MessageRole.ASSISTANT, content="assistant"),
            )
        )

        await model.generate(request)

        call = generate_content.await_args.kwargs
        self.assertEqual(call["model"], "gemini-3.5-flash")
        self.assertEqual(call["config"].system_instruction, ["system one", "system two"])
        self.assertEqual([content.role for content in call["contents"]], ["user", "model"])
        self.assertEqual(call["contents"][0].parts[0].text, "user")
        self.assertEqual(call["contents"][1].parts[0].text, "assistant")

    async def test_request_model_and_generation_parameters_override_defaults(self) -> None:
        model, generate_content = self.make_model(response=make_response())
        request = ChatRequest(
            messages=(ChatMessage(role=MessageRole.USER, content="hello"),),
            model="gemini-requested",
            temperature=0.25,
            max_output_tokens=321,
        )

        await model.generate(request)

        call = generate_content.await_args.kwargs
        self.assertEqual(call["model"], "gemini-requested")
        self.assertEqual(call["config"].temperature, 0.25)
        self.assertEqual(call["config"].max_output_tokens, 321)

    async def test_rejects_named_messages_and_system_only_requests(self) -> None:
        cases = (
            ChatRequest(
                messages=(
                    ChatMessage(
                        role=MessageRole.USER,
                        content="hello",
                        name="customer",
                    ),
                )
            ),
            ChatRequest(messages=(ChatMessage(role=MessageRole.SYSTEM, content="system"),)),
        )

        for request in cases:
            with self.subTest(request=request):
                model, generate_content = self.make_model(response=make_response())
                with self.assertRaises(InvalidRequestError):
                    await model.generate(request)
                generate_content.assert_not_awaited()

    async def test_normalizes_text_finish_reason_model_and_usage(self) -> None:
        model, _ = self.make_model(response=make_response())

        response = await model.generate(
            ChatRequest(messages=(ChatMessage(role=MessageRole.USER, content="hello"),))
        )

        self.assertEqual(response.message.content, "hello from Gemini")
        self.assertEqual(response.message.role, MessageRole.ASSISTANT)
        self.assertEqual(response.provider, "gemini")
        self.assertEqual(response.model, "gemini-test-response")
        self.assertEqual(response.finish_reason, "stop")
        self.assertEqual(response.usage.input_tokens, 10)
        self.assertEqual(response.usage.output_tokens, 7)
        self.assertEqual(response.usage.total_tokens, 17)

    async def test_uses_candidate_and_thought_counts_when_total_is_missing(self) -> None:
        model, _ = self.make_model(response=make_response(total_tokens=None))

        response = await model.generate(
            ChatRequest(messages=(ChatMessage(role=MessageRole.USER, content="hello"),))
        )

        self.assertEqual(response.usage.input_tokens, 10)
        self.assertEqual(response.usage.output_tokens, 7)

    async def test_defaults_missing_optional_response_metadata(self) -> None:
        sdk_response = make_response(model="gemini-test")
        sdk_response.model_version = None
        sdk_response.candidates[0].finish_reason = None
        sdk_response.usage_metadata = None
        model, _ = self.make_model(response=sdk_response)

        response = await model.generate(
            ChatRequest(messages=(ChatMessage(role=MessageRole.USER, content="hello"),))
        )

        self.assertEqual(response.model, "gemini-3.5-flash")
        self.assertEqual(response.finish_reason, "unknown")
        self.assertEqual(response.usage.total_tokens, 0)

    async def test_rejects_empty_or_malformed_responses(self) -> None:
        responses = (
            None,
            types.GenerateContentResponse(candidates=[]),
            make_response(content=" "),
            SimpleNamespace(model_version="gemini-test"),
        )

        for sdk_response in responses:
            with self.subTest(response=sdk_response):
                model, _ = self.make_model(response=sdk_response)
                with self.assertRaises(ProviderResponseError):
                    await model.generate(
                        ChatRequest(messages=(ChatMessage(role=MessageRole.USER, content="hello"),))
                    )

    async def test_maps_google_api_errors_and_preserves_safe_metadata(self) -> None:
        api_key_value = "gemini-test-key"
        cases = (
            (
                make_api_error(errors.ClientError, 401, "UNAUTHENTICATED", api_key_value),
                ProviderAuthenticationError,
            ),
            (
                make_api_error(errors.ClientError, 403, "PERMISSION_DENIED", api_key_value),
                ProviderPermissionDeniedError,
            ),
            (
                make_api_error(errors.ClientError, 408, "DEADLINE_EXCEEDED", api_key_value),
                ProviderTimeoutError,
            ),
            (
                make_api_error(errors.ClientError, 429, "RESOURCE_EXHAUSTED", api_key_value),
                ProviderRateLimitedError,
            ),
            (
                make_api_error(errors.ClientError, 400, "INVALID_ARGUMENT", api_key_value),
                InvalidRequestError,
            ),
            (
                make_api_error(errors.ServerError, 503, "UNAVAILABLE", api_key_value),
                ProviderUnavailableError,
            ),
        )

        for sdk_error, expected_error in cases:
            with self.subTest(status=sdk_error.code):
                model, _ = self.make_model(error=sdk_error)
                with (
                    self.assertNoLogs("provider.gemini_chat", level="DEBUG"),
                    self.assertRaises(expected_error) as raised,
                ):
                    await model.generate(
                        ChatRequest(
                            messages=(
                                ChatMessage(
                                    role=MessageRole.USER,
                                    content=f"prompt {api_key_value}",
                                ),
                            )
                        )
                    )

                self.assertEqual(raised.exception.metadata.status_code, sdk_error.code)
                self.assertEqual(raised.exception.metadata.error_code, sdk_error.status)
                self.assertEqual(raised.exception.metadata.request_id, "req_gemini123")
                rendered_error = "".join(traceback.format_exception(raised.exception))
                self.assertNotIn(api_key_value, rendered_error)

    async def test_maps_http_transport_errors(self) -> None:
        request = httpx.Request("POST", "https://generativelanguage.googleapis.com")
        timeout_marker = "timeout-leak-marker"
        connection_marker = "connection-leak-marker"
        cases = (
            (
                httpx.ReadTimeout(timeout_marker, request=request),
                ProviderTimeoutError,
                timeout_marker,
            ),
            (
                httpx.ConnectError(connection_marker, request=request),
                ProviderUnavailableError,
                connection_marker,
            ),
        )

        for transport_error, expected_error, marker in cases:
            with self.subTest(error=type(transport_error).__name__):
                model, _ = self.make_model(error=transport_error)
                with self.assertRaises(expected_error) as raised:
                    await model.generate(
                        ChatRequest(messages=(ChatMessage(role=MessageRole.USER, content="hello"),))
                    )
                self.assertNotIn(marker, str(raised.exception))

    async def test_streams_text_deltas_and_final_metadata(self) -> None:
        first = make_response(
            content="hello ",
            model="gemini-stream-response",
            finish_reason=None,
        )
        first.usage_metadata = None
        final = make_response(
            content="world",
            model="gemini-stream-response",
            finish_reason=types.FinishReason.STOP,
        )
        model, generate_content_stream = self.make_stream_model(responses=(first, final))
        request = ChatRequest(
            messages=(ChatMessage(role=MessageRole.USER, content="hello"),),
            model="gemini-requested",
            temperature=0.25,
            max_output_tokens=321,
        )

        chunks = [chunk async for chunk in model.stream(request)]

        self.assertEqual("".join(chunk.text_delta for chunk in chunks), "hello world")
        self.assertEqual(chunks[-1].provider, "gemini")
        self.assertEqual(chunks[-1].model, "gemini-stream-response")
        self.assertEqual(chunks[-1].finish_reason, "stop")
        self.assertEqual(chunks[-1].usage.total_tokens, 17)
        call = generate_content_stream.await_args.kwargs
        self.assertEqual(call["model"], "gemini-requested")
        self.assertEqual(call["config"].temperature, 0.25)
        self.assertEqual(call["config"].max_output_tokens, 321)

    async def test_rejects_stream_without_text(self) -> None:
        model, _ = self.make_stream_model()

        with self.assertRaisesRegex(ProviderResponseError, "no streamed text"):
            _ = [
                chunk
                async for chunk in model.stream(
                    ChatRequest(messages=(ChatMessage(role=MessageRole.USER, content="hello"),))
                )
            ]

    async def test_maps_stream_start_errors(self) -> None:
        request = httpx.Request("POST", "https://generativelanguage.googleapis.com")
        cases = (
            (
                make_api_error(
                    errors.ClientError,
                    429,
                    "RESOURCE_EXHAUSTED",
                    "provider-message-marker",
                ),
                ProviderRateLimitedError,
            ),
            (
                httpx.ReadTimeout("timeout-leak-marker-2", request=request),
                ProviderTimeoutError,
            ),
        )

        for stream_error, expected_error in cases:
            with self.subTest(error=type(stream_error).__name__):
                model, _ = self.make_stream_model(error=stream_error)
                with self.assertRaises(expected_error) as raised:
                    _ = [
                        chunk
                        async for chunk in model.stream(
                            ChatRequest(
                                messages=(ChatMessage(role=MessageRole.USER, content="hello"),)
                            )
                        )
                    ]
                self.assertNotIn("timeout-leak-marker-2", str(raised.exception))

    def test_declares_chat_and_streaming_capabilities_and_registers(self) -> None:
        model, _ = self.make_model(response=make_response())
        registry = ProviderRegistry()

        registry.register(model)

        self.assertIsInstance(model, ChatModel)
        self.assertIsInstance(model, StreamingChatModel)
        self.assertEqual(model.provider_id, "gemini")
        self.assertEqual(
            model.capabilities,
            frozenset({ModelCapability.CHAT, ModelCapability.STREAMING}),
        )
        self.assertIs(registry.resolve("gemini"), model)


if __name__ == "__main__":
    unittest.main()

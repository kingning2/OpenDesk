"""Offline tests for the OpenAI Chat Completions adapter."""

from __future__ import annotations

import traceback
import unittest
from types import SimpleNamespace
from unittest.mock import AsyncMock, patch

import httpx
from openai import (
    APIConnectionError,
    APIStatusError,
    APITimeoutError,
    AuthenticationError,
    BadRequestError,
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
from model import ChatMessage, ChatRequest, MessageRole, ModelCapability
from provider import OpenAIChatConfig, OpenAIChatModel, ProviderRegistry


def make_response(
    *,
    content: str = "hello",
    model: str = "gpt-test-response",
    finish_reason: str = "stop",
    prompt_tokens: int = 7,
    completion_tokens: int = 3,
) -> SimpleNamespace:
    return SimpleNamespace(
        choices=[
            SimpleNamespace(
                message=SimpleNamespace(content=content),
                finish_reason=finish_reason,
            )
        ],
        model=model,
        usage=SimpleNamespace(
            prompt_tokens=prompt_tokens,
            completion_tokens=completion_tokens,
        ),
    )


def make_client(
    *, response: object | None = None, error: Exception | None = None
) -> tuple[SimpleNamespace, AsyncMock]:
    create = AsyncMock(side_effect=error) if error is not None else AsyncMock(return_value=response)
    client = SimpleNamespace(chat=SimpleNamespace(completions=SimpleNamespace(create=create)))
    return client, create


def make_status_error(
    error_type: type[APIStatusError],
    status_code: int,
    message: str,
    *,
    error_code: str = "test_error",
    request_id: str = "req_test123",
) -> APIStatusError:
    request = httpx.Request("POST", "https://provider.invalid/v1/chat/completions")
    response = httpx.Response(
        status_code,
        request=request,
        headers={"x-request-id": request_id},
    )
    return error_type(
        message,
        response=response,
        body={"error": {"message": message, "code": error_code}},
    )


class OpenAIChatConfigTests(unittest.TestCase):
    def test_defaults_and_secret_repr(self) -> None:
        api_key_value = "sk-sensitive-value"
        config = OpenAIChatConfig(model="gpt-test", api_key=api_key_value)

        self.assertEqual(config.base_url, None)
        self.assertEqual(config.timeout_seconds, 60.0)
        self.assertEqual(config.max_retries, 2)
        self.assertNotIn(api_key_value, repr(config))
        self.assertNotIn("api_key", repr(config))

    def test_rejects_invalid_configuration_without_echoing_secret(self) -> None:
        api_key_value = "sk-sensitive-value"
        cases = (
            {"model": "", "api_key": api_key_value},
            {"model": "gpt-test", "api_key": ""},
            {
                "model": "gpt-test",
                "api_key": api_key_value,
                "base_url": " ",
            },
            {
                "model": "gpt-test",
                "api_key": api_key_value,
                "timeout_seconds": 0,
            },
            {
                "model": "gpt-test",
                "api_key": api_key_value,
                "max_retries": -1,
            },
        )

        for options in cases:
            with self.subTest(options=options):
                with self.assertRaises(ProviderConfigurationError) as raised:
                    OpenAIChatConfig(**options)
                self.assertNotIn(api_key_value, str(raised.exception))

    def test_builds_official_client_from_explicit_configuration(self) -> None:
        api_key_value = "sk-sensitive-value"
        config = OpenAIChatConfig(
            model="gpt-test",
            api_key=api_key_value,
            base_url="https://provider.invalid/v1",
            timeout_seconds=12.5,
            max_retries=4,
        )

        with patch("provider.openai_chat.AsyncOpenAI") as constructor:
            OpenAIChatModel(config)

        constructor.assert_called_once_with(
            api_key=api_key_value,
            base_url="https://provider.invalid/v1",
            timeout=12.5,
            max_retries=4,
        )


class OpenAIChatModelTests(unittest.IsolatedAsyncioTestCase):
    def make_model(
        self, *, response: object | None = None, error: Exception | None = None
    ) -> tuple[OpenAIChatModel, AsyncMock]:
        client, create = make_client(response=response, error=error)
        api_key_value = "sk-sensitive-value"
        config = OpenAIChatConfig(model="gpt-default", api_key=api_key_value)
        return OpenAIChatModel(config, client=client), create

    async def test_converts_all_supported_roles_and_optional_name(self) -> None:
        model, create = self.make_model(response=make_response())
        request = ChatRequest(
            messages=(
                ChatMessage(role=MessageRole.SYSTEM, content="system"),
                ChatMessage(role=MessageRole.USER, content="user", name="customer"),
                ChatMessage(role=MessageRole.ASSISTANT, content="assistant"),
            )
        )

        await model.generate(request)

        self.assertEqual(
            create.await_args.kwargs["messages"],
            [
                {"role": "system", "content": "system"},
                {"role": "user", "content": "user", "name": "customer"},
                {"role": "assistant", "content": "assistant"},
            ],
        )

    async def test_uses_default_model_and_omits_unset_optional_parameters(self) -> None:
        model, create = self.make_model(response=make_response())

        await model.generate(
            ChatRequest(messages=(ChatMessage(role=MessageRole.USER, content="hello"),))
        )

        self.assertEqual(create.await_args.kwargs["model"], "gpt-default")
        self.assertFalse(create.await_args.kwargs["stream"])
        self.assertNotIn("temperature", create.await_args.kwargs)
        self.assertNotIn("max_completion_tokens", create.await_args.kwargs)

    async def test_request_model_and_generation_parameters_override_defaults(self) -> None:
        model, create = self.make_model(response=make_response())
        request = ChatRequest(
            messages=(ChatMessage(role=MessageRole.USER, content="hello"),),
            model="gpt-requested",
            temperature=0.25,
            max_output_tokens=321,
        )

        await model.generate(request)

        self.assertEqual(create.await_args.kwargs["model"], "gpt-requested")
        self.assertEqual(create.await_args.kwargs["temperature"], 0.25)
        self.assertEqual(create.await_args.kwargs["max_completion_tokens"], 321)

    async def test_normalizes_text_finish_reason_model_and_usage(self) -> None:
        model, _ = self.make_model(
            response=make_response(
                content="normalized response",
                model="gpt-actual",
                finish_reason="length",
                prompt_tokens=11,
                completion_tokens=5,
            )
        )

        response = await model.generate(
            ChatRequest(messages=(ChatMessage(role=MessageRole.USER, content="hello"),))
        )

        self.assertEqual(response.message.content, "normalized response")
        self.assertEqual(response.message.role, MessageRole.ASSISTANT)
        self.assertEqual(response.provider, "openai")
        self.assertEqual(response.model, "gpt-actual")
        self.assertEqual(response.finish_reason, "length")
        self.assertEqual(response.usage.input_tokens, 11)
        self.assertEqual(response.usage.output_tokens, 5)
        self.assertEqual(response.usage.total_tokens, 16)

    async def test_defaults_missing_optional_response_metadata(self) -> None:
        sdk_response = make_response()
        sdk_response.model = None
        sdk_response.choices[0].finish_reason = None
        sdk_response.usage = None
        model, _ = self.make_model(response=sdk_response)

        response = await model.generate(
            ChatRequest(messages=(ChatMessage(role=MessageRole.USER, content="hello"),))
        )

        self.assertEqual(response.model, "gpt-default")
        self.assertEqual(response.finish_reason, "unknown")
        self.assertEqual(response.usage.total_tokens, 0)

    async def test_rejects_empty_or_malformed_responses(self) -> None:
        responses = (
            None,
            SimpleNamespace(choices=[]),
            make_response(content=" "),
            SimpleNamespace(model="gpt-test"),
        )

        for sdk_response in responses:
            with self.subTest(response=sdk_response):
                model, _ = self.make_model(response=sdk_response)
                with self.assertRaises(ProviderResponseError):
                    await model.generate(
                        ChatRequest(messages=(ChatMessage(role=MessageRole.USER, content="hello"),))
                    )

    async def test_maps_official_sdk_errors_to_stable_errors(self) -> None:
        api_key_value = "sk-sensitive-value"
        request = httpx.Request("POST", "https://provider.invalid/v1/chat/completions")
        cases = (
            (
                make_status_error(AuthenticationError, 401, f"bad key {api_key_value}"),
                ProviderAuthenticationError,
            ),
            (
                make_status_error(RateLimitError, 429, f"rate body {api_key_value}"),
                ProviderRateLimitedError,
            ),
            (APITimeoutError(request), ProviderTimeoutError),
            (
                APIConnectionError(message=f"connection body {api_key_value}", request=request),
                ProviderUnavailableError,
            ),
            (
                make_status_error(BadRequestError, 400, f"prompt body {api_key_value}"),
                InvalidRequestError,
            ),
            (
                make_status_error(APIStatusError, 503, f"server body {api_key_value}"),
                ProviderUnavailableError,
            ),
        )

        for sdk_error, expected_error in cases:
            with self.subTest(sdk_error=type(sdk_error).__name__):
                model, _ = self.make_model(error=sdk_error)
                with (
                    self.assertNoLogs("provider.openai_chat", level="DEBUG"),
                    self.assertRaises(expected_error) as raised,
                ):
                    await model.generate(
                        ChatRequest(
                            messages=(
                                ChatMessage(
                                    role=MessageRole.USER,
                                    content=f"private prompt {api_key_value}",
                                ),
                            )
                        )
                    )

                rendered_error = "".join(traceback.format_exception(raised.exception))
                self.assertNotIn(api_key_value, str(raised.exception))
                self.assertNotIn(api_key_value, rendered_error)

    async def test_maps_permission_denied_and_preserves_safe_metadata(self) -> None:
        api_key_value = "sk-sensitive-value"
        sdk_error = make_status_error(
            PermissionDeniedError,
            403,
            f"Project cannot access model; private body {api_key_value}",
            error_code="model_not_found",
            request_id="req_permission123",
        )
        model, _ = self.make_model(error=sdk_error)

        with self.assertRaises(ProviderPermissionDeniedError) as raised:
            await model.generate(
                ChatRequest(
                    messages=(
                        ChatMessage(
                            role=MessageRole.USER,
                            content=f"private prompt {api_key_value}",
                        ),
                    )
                )
            )

        self.assertEqual(
            raised.exception.metadata,
            ProviderErrorMetadata(
                status_code=403,
                error_code="model_not_found",
                request_id="req_permission123",
            ),
        )
        rendered_error = "".join(traceback.format_exception(raised.exception))
        self.assertIn("provider_error_code=model_not_found", rendered_error)
        self.assertIn("request_id=req_permission123", rendered_error)
        self.assertNotIn(api_key_value, rendered_error)
        self.assertNotIn("Project cannot access model", rendered_error)

    async def test_drops_unsafe_provider_metadata(self) -> None:
        api_key_value = "sk-sensitive-value"
        sdk_error = make_status_error(
            PermissionDeniedError,
            403,
            "permission denied",
            error_code=f"raw body {api_key_value}",
            request_id="req_safe123",
        )
        model, _ = self.make_model(error=sdk_error)

        with self.assertRaises(ProviderPermissionDeniedError) as raised:
            await model.generate(
                ChatRequest(messages=(ChatMessage(role=MessageRole.USER, content="hello"),))
            )

        self.assertIsNone(raised.exception.metadata.error_code)
        self.assertEqual(raised.exception.metadata.request_id, "req_safe123")
        self.assertNotIn(api_key_value, str(raised.exception))

    def test_declares_only_chat_capability_and_registers(self) -> None:
        model, _ = self.make_model(response=make_response())
        registry = ProviderRegistry()

        registry.register(model)

        self.assertIsInstance(model, ChatModel)
        self.assertEqual(model.provider_id, "openai")
        self.assertEqual(model.capabilities, frozenset({ModelCapability.CHAT}))
        self.assertIs(registry.resolve("openai"), model)


if __name__ == "__main__":
    unittest.main()

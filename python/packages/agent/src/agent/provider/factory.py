"""LLM Provider 工厂：按厂商路由到 vendors 实现。

作者：Xiaoman
创建时间：2026-07-22
"""

from __future__ import annotations

import logging

from agent.provider.base import LlmProvider
from agent.provider.config import LlmProviderConfig
from agent.provider.vendors import (
    AnthropicProvider,
    DeepSeekProvider,
    DoubaoProvider,
    KimiProvider,
    OllamaProvider,
    OpenAiCompatibleProvider,
    OpenAiProvider,
)

logger = logging.getLogger("opendesk.sidecar.llm")


def create_provider(config: LlmProviderConfig) -> LlmProvider:
    """根据配置构造对应厂商 Provider。

    作者：Xiaoman
    创建时间：2026-07-22

    Args:
        config: 内存态 Provider 配置。

    Returns:
        具体厂商 Provider 实例。
    """
    kind = config.provider.strip().lower()
    if kind == "openai":
        return OpenAiProvider(config)
    if kind == "anthropic":
        return AnthropicProvider(config)
    if kind == "openai_compatible":
        return _resolve_openai_compatible(config)
    if kind == "deepseek":
        return DeepSeekProvider(config)
    if kind in {"doubao", "ark", "volcengine"}:
        return DoubaoProvider(config)
    if kind in {"kimi", "moonshot"}:
        return KimiProvider(config)
    if kind == "ollama":
        return OllamaProvider(config)
    logger.warning(
        "unknown llm provider, fallback openai_compatible",
        extra={"event": "llm.unknown_provider", "provider": config.provider},
    )
    return OpenAiCompatibleProvider(config)


def _resolve_openai_compatible(config: LlmProviderConfig) -> LlmProvider:
    """按 Base URL 推断 DeepSeek / 豆包 / Kimi / Ollama / 自定义。

    作者：Xiaoman
    创建时间：2026-07-22
    """
    base = (config.base_url or "").strip().lower()
    if "deepseek.com" in base:
        return DeepSeekProvider(config)
    if "volces.com" in base or "ark.cn-beijing" in base:
        return DoubaoProvider(config)
    if "moonshot." in base:
        return KimiProvider(config)
    if "11434" in base or "ollama" in base:
        return OllamaProvider(config)
    return OpenAiCompatibleProvider(config)


def test_connection(config: LlmProviderConfig) -> tuple[bool, str | None, str]:
    """对当前配置做连通性探测。

    作者：Xiaoman
    创建时间：2026-07-22

    Args:
        config: 内存态 Provider 配置。

    Returns:
        ``(ok, error_code, message)``。
    """
    try:
        return create_provider(config).test_connection()
    except Exception as exc:  # noqa: BLE001
        logger.warning(
            "llm test_connection failed",
            extra={"event": "llm.test_failed", "feature": "runtime"},
        )
        return False, "LLM_TEST_FAILED", str(exc)

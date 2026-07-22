"""Ollama 本机 Provider（OpenAI 兼容 SDK）。

作者：Xiaoman
创建时间：2026-07-22
"""

from __future__ import annotations

from agent.provider.base import build_openai_client, probe_openai_models
from agent.provider.config import LlmProviderConfig

DEFAULT_BASE = "".join(["http://", "127.0.0.1", ":11434/v1"])


class OllamaProvider:
    """本机 Ollama OpenAI 兼容接入（API Key 可选）。

    作者：Xiaoman
    创建时间：2026-07-22
    """

    def __init__(self, config: LlmProviderConfig) -> None:
        """初始化。

        Args:
            config: 内存态 Provider 配置。
        """
        self._config = config

    def test_connection(self) -> tuple[bool, str | None, str]:
        """调用 Ollama ``/v1/models`` 探测连通性。

        Returns:
            ``(ok, error_code, message)``。
        """
        client = build_openai_client(
            api_key=self._config.api_key,
            base_url=self._config.base_url,
            default_base=DEFAULT_BASE,
            fallback_key="ollama",
        )
        return probe_openai_models(client)

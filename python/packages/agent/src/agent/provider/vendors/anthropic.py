"""Anthropic Provider（anthropic SDK）。

作者：Xiaoman
创建时间：2026-07-22
"""

from __future__ import annotations

from anthropic import Anthropic

from agent.provider.base import require_api_key
from agent.provider.config import LlmProviderConfig

DEFAULT_BASE = "https://api.anthropic.com"


class AnthropicProvider:
    """Anthropic Claude 接入。

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
        """调用 Anthropic ``models.list`` 探测连通性。

        Returns:
            ``(ok, error_code, message)``。
        """
        missing = require_api_key(self._config)
        if missing is not None:
            return missing
        base = (self._config.base_url or DEFAULT_BASE).rstrip("/")
        try:
            client = Anthropic(
                api_key=self._config.api_key,
                base_url=base,
                timeout=20.0,
            )
            page = client.models.list()
            _ = next(iter(page), None)
            return True, None, "ok"
        except Exception as exc:  # noqa: BLE001
            return False, "LLM_TEST_FAILED", str(exc)

"""DeepSeek Provider（OpenAI 兼容 SDK）。

作者：Xiaoman
创建时间：2026-07-22
"""

from __future__ import annotations

from agent.provider.base import (
    build_openai_client,
    probe_openai_models,
    require_api_key,
)
from agent.provider.config import LlmProviderConfig

DEFAULT_BASE = "https://api.deepseek.com"


class DeepSeekProvider:
    """DeepSeek OpenAI 兼容接入。

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
        """调用 DeepSeek ``models.list`` 探测连通性。

        Returns:
            ``(ok, error_code, message)``。
        """
        missing = require_api_key(self._config)
        if missing is not None:
            return missing
        client = build_openai_client(
            api_key=self._config.api_key,
            base_url=self._config.base_url,
            default_base=DEFAULT_BASE,
        )
        return probe_openai_models(client)

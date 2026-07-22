"""OpenAI 官方 Provider（openai SDK）。

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

DEFAULT_BASE = "https://api.openai.com/v1"


class OpenAiProvider:
    """OpenAI 官方模型接入。

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
        """调用 OpenAI ``models.list`` 探测连通性。

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

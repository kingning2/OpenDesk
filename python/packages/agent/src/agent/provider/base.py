"""LLM Provider 公共协议与 OpenAI 兼容客户端辅助。

作者：Xiaoman
创建时间：2026-07-22
"""

from __future__ import annotations

import re
from typing import Protocol

from openai import OpenAI

from agent.provider.config import LlmProviderConfig

_VERSIONED_BASE_RE = re.compile(r"/v\d+$")


class LlmProvider(Protocol):
    """各厂商 Provider 最小接口。

    作者：Xiaoman
    创建时间：2026-07-22
    """

    def test_connection(self) -> tuple[bool, str | None, str]:
        """轻量连通性探测（优先 models.list）。

        Returns:
            ``(ok, error_code, message)``。
        """
        ...


def normalize_openai_base(base_url: str | None, default: str) -> str:
    """规范化 OpenAI 兼容 Base URL。

    已带 ``/v1``、``/v3`` 等版本后缀时不再追加 ``/v1``。

    作者：Xiaoman
    创建时间：2026-07-22

    Args:
        base_url: 用户配置的 Base URL。
        default: 厂商默认 Base URL。

    Returns:
        规范化后的 Base URL。
    """
    base = (base_url or default).rstrip("/")
    if _VERSIONED_BASE_RE.search(base):
        return base
    return f"{base}/v1"


def build_openai_client(
    *,
    api_key: str,
    base_url: str | None,
    default_base: str,
    fallback_key: str = "opendesk",
) -> OpenAI:
    """构造 OpenAI SDK 客户端（兼容 DeepSeek / 豆包 / Kimi / Ollama）。

    作者：Xiaoman
    创建时间：2026-07-22

    Args:
        api_key: API Key；空串时用 ``fallback_key``（Ollama 等本地场景）。
        base_url: 可选 Base URL。
        default_base: 厂商默认 Base URL。
        fallback_key: 空 key 时的占位（SDK 要求非空字符串）。

    Returns:
        ``OpenAI`` 客户端实例。
    """
    return OpenAI(
        api_key=api_key.strip() or fallback_key,
        base_url=normalize_openai_base(base_url, default_base),
        timeout=20.0,
    )


def probe_openai_models(client: OpenAI) -> tuple[bool, str | None, str]:
    """用 OpenAI SDK ``models.list`` 探测连通性。

    作者：Xiaoman
    创建时间：2026-07-22

    Args:
        client: OpenAI SDK 客户端。

    Returns:
        ``(ok, error_code, message)``。
    """
    try:
        page = client.models.list()
        # 触发一次实际迭代，避免惰性对象未发请求。
        _ = next(iter(page), None)
        return True, None, "ok"
    except Exception as exc:  # noqa: BLE001 — 统一映射为 LLM_TEST_FAILED
        return False, "LLM_TEST_FAILED", str(exc)


def require_api_key(config: LlmProviderConfig) -> tuple[bool, str | None, str] | None:
    """若缺少 API Key 则返回未配置错误；否则返回 ``None`` 表示可继续。

    作者：Xiaoman
    创建时间：2026-07-22
    """
    if not config.api_key.strip():
        return False, "LLM_NOT_CONFIGURED", "API key is required"
    return None

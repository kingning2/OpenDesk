"""LLM Provider 配置（由 Rust 经环境变量 / 请求 payload 注入）。

作者：Xiaoman
创建时间：2026-07-22
"""

from __future__ import annotations

import os
from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class LlmProviderConfig:
    """内存态 Provider 配置（不含持久化）。

    作者：Xiaoman
    创建时间：2026-07-22
    """

    provider: str
    model_id: str
    api_key: str = ""
    base_url: str | None = None


def from_env() -> LlmProviderConfig | None:
    """从 Rust 注入的环境变量读取配置。

    作者：Xiaoman
    创建时间：2026-07-22

    Returns:
        已配置时返回 ``LlmProviderConfig``；缺少 provider/model 时返回 ``None``。
    """
    provider = os.environ.get("OPENDESK_LLM_PROVIDER", "").strip()
    model_id = os.environ.get("OPENDESK_LLM_MODEL_ID", "").strip()
    if not provider or not model_id:
        return None
    base_url = os.environ.get("OPENDESK_LLM_BASE_URL", "").strip() or None
    api_key = os.environ.get("OPENDESK_LLM_API_KEY", "")
    return LlmProviderConfig(
        provider=provider,
        model_id=model_id,
        api_key=api_key,
        base_url=base_url,
    )


def from_payload(payload: dict[str, object] | None) -> LlmProviderConfig | None:
    """从 Sidecar 请求体构造配置。

    作者：Xiaoman
    创建时间：2026-07-22

    Args:
        payload: Sidecar JSON 请求体。

    Returns:
        合法时返回配置；否则 ``None``。
    """
    if not isinstance(payload, dict):
        return None
    provider = str(payload.get("provider", "")).strip()
    model_id = str(payload.get("model_id", "")).strip()
    if not provider or not model_id:
        return None
    raw_base = payload.get("base_url")
    base_url = str(raw_base).strip() if raw_base else None
    if base_url == "":
        base_url = None
    api_key = str(payload.get("api_key", ""))
    return LlmProviderConfig(
        provider=provider,
        model_id=model_id,
        api_key=api_key,
        base_url=base_url,
    )

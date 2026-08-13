"""Sidecar handler: /v1/llm/chat (POST) — Python ← Rust only.

纯大模型接入：接收 OpenAI 兼容 provider 配置与消息列表，调用模型返回回复。
不含任何业务/渠道逻辑（渠道调度在 Rust 侧）。
"""

from __future__ import annotations

import logging
import time
from typing import Any

from shared.logging import bind_log_context

logger = logging.getLogger("opendesk.sidecar.llm")

try:  # openai 为可选运行时依赖；缺失时返回可读错误。
    from openai import OpenAI
except ImportError:  # pragma: no cover
    OpenAI = None  # type: ignore[assignment]


def handle_llm_chat(payload: dict[str, Any] | None, *, trace_id: str) -> dict[str, Any]:
    """Contract: contracts/schema/v1/llm/ipc/chat.request/response.schema.json"""
    with bind_log_context(trace_id=trace_id, feature="llm"):
        if payload is None:
            return {"reply": "", "trace_id": trace_id}
        messages = payload.get("messages") or []
        provider = payload.get("provider") or {}

        if OpenAI is None:
            logger.warning("openai 未安装，无法生成回复", extra={"event": "llm.chat.missing_dep"})
            return {
                "reply": "LLM 依赖未安装：请运行 uv add openai",
                "trace_id": trace_id,
            }

        base_url = provider.get("base_url") or ""
        api_key = provider.get("api_key") or ""
        model = provider.get("model") or ""

        if not base_url or not api_key or not model:
            logger.warning("provider 配置不完整", extra={"event": "llm.chat.invalid_config"})
            return {
                "reply": "LLM provider 未配置完整（base_url/api_key/model）",
                "trace_id": trace_id,
            }

        started = time.perf_counter()
        try:
            client = OpenAI(base_url=base_url, api_key=api_key)
            completion = client.chat.completions.create(
                model=model,
                messages=messages,
            )
            reply = completion.choices[0].message.content or ""
            duration_ms = max(0, int((time.perf_counter() - started) * 1000))
            logger.info(
                "LLM 对话成功 model=%s messages=%s reply_chars=%s duration_ms=%s",
                model,
                len(messages),
                len(reply),
                duration_ms,
                extra={
                    "event": "llm.chat.ok",
                    "model": model,
                    "message_count": len(messages),
                    "reply_chars": len(reply),
                    "duration_ms": duration_ms,
                },
            )
            return {"reply": reply, "trace_id": trace_id}
        except Exception as _error:  # noqa: BLE001
            duration_ms = max(0, int((time.perf_counter() - started) * 1000))
            logger.exception(
                "LLM 对话失败 model=%s duration_ms=%s",
                model,
                duration_ms,
                extra={
                    "event": "llm.chat.failed",
                    "model": model,
                    "duration_ms": duration_ms,
                },
            )
            return {
                "reply": f"LLM 调用失败：{_error}",
                "trace_id": trace_id,
            }

"""Sidecar handler: /v1/runtime/llm_test_connection (POST) — Python ← Rust only.

作者：Xiaoman
创建时间：2026-07-22
"""

from __future__ import annotations

import logging
from typing import Any

from agent.provider import from_payload, test_connection
from shared.logging import bind_log_context

logger = logging.getLogger("opendesk.sidecar.runtime")


def handle_llm_test_connection(
    payload: dict[str, Any] | None,
    *,
    trace_id: str,
) -> dict[str, Any]:
    """Contract: contracts/schema/v1/runtime/sidecar/llm_test_connection.*.schema.json

    作者：Xiaoman
    创建时间：2026-07-22

    Args:
        payload: Sidecar 请求体（含内存态 api_key）。
        trace_id: 追踪 ID。

    Returns:
        探测结果字典；永不回显 api_key。
    """
    with bind_log_context(trace_id=trace_id, feature="runtime"):
        config = from_payload(payload)
        if config is None:
            logger.info(
                "llm_test_connection not configured",
                extra={"event": "llm.not_configured"},
            )
            return {
                "ok": False,
                "error_code": "LLM_NOT_CONFIGURED",
                "message": "Provider and model_id are required",
                "trace_id": trace_id,
            }
        logger.info(
            "llm_test_connection start",
            extra={
                "event": "llm.test_start",
                "provider": config.provider,
                "model_id": config.model_id,
                "has_api_key": bool(config.api_key),
            },
        )
        ok, error_code, message = test_connection(config)
        return {
            "ok": ok,
            "error_code": error_code,
            "message": message,
            "trace_id": trace_id,
        }

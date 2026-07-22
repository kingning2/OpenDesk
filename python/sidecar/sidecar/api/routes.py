"""Sidecar 路由表（路径 → HTTP 方法 + Handler 名）。

作者：Xiaoman
创建时间：2026-07-22
"""

from __future__ import annotations

ROUTES: dict[str, tuple[str, str]] = {
    "/v1/agent/ping": ("POST", "handle_agent_ping"),
    "/v1/runtime/llm_test_connection": ("POST", "handle_llm_test_connection"),
}

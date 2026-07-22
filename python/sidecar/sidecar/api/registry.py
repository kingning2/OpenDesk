"""Sidecar 业务 Handler 注册表（名称 → 可调用对象）。

作者：Xiaoman
创建时间：2026-07-22
"""

from __future__ import annotations

from collections.abc import Callable
from typing import Any

from gateway.handlers import handle_agent_ping, handle_llm_test_connection

HandlerFn = Callable[..., dict[str, Any]]

HANDLERS: dict[str, HandlerFn] = {
    "handle_agent_ping": handle_agent_ping,
    "handle_llm_test_connection": handle_llm_test_connection,
}

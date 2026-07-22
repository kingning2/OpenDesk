"""Auto-generated from contracts/schema."""

from typing import TypedDict


class RuntimeSidecarLlmTestConnectionResponse(TypedDict, total=False):
    ok: bool
    error_code: str
    message: str
    trace_id: str

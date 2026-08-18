"""Example: Agent task handler — output returns to Rust, not React."""

from __future__ import annotations

from collections.abc import Iterator


def stream_tokens(task_id: str) -> Iterator[str]:
    """Rust consumes this iterator and forwards via Tauri Events."""
    yield from ()  # skeleton — no business tokens

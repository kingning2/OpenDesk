"""Sidecar compatibility exports for shared structured logging."""

from __future__ import annotations

from shared.logging import JsonLineFormatter, configure_logging

JsonLogFormatter = JsonLineFormatter

__all__ = ["JsonLogFormatter", "configure_logging"]

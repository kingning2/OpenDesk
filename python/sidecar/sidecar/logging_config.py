"""Structured JSON logging for Rust-side log pipe parsing."""

from __future__ import annotations

import json
import logging
import sys
from typing import Any

_STRUCTURED_KEYS = ("trace_id", "task_id", "feature", "tenant_id")


class JsonLogFormatter(logging.Formatter):
    def format(self, record: logging.LogRecord) -> str:
        payload: dict[str, Any] = {
            "level": record.levelname,
            "message": record.getMessage(),
            "logger": record.name,
        }
        for key in _STRUCTURED_KEYS:
            value = getattr(record, key, None)
            if value is not None:
                payload[key] = value
        return json.dumps(payload, ensure_ascii=False)


def configure_logging(level: int = logging.INFO) -> None:
    handler = logging.StreamHandler(sys.stdout)
    handler.setFormatter(JsonLogFormatter())
    logging.basicConfig(level=level, handlers=[handler], force=True)

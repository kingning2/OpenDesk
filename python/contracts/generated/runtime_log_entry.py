"""Auto-generated from contracts/schema."""

from typing import TypedDict


class RuntimeLogEntry(TypedDict, total=False):
    schema_version: str
    timestamp: str
    level: str
    source: str
    logger: str
    message: str
    event: str
    feature: str
    trace_id: str
    task_id: str
    tenant_id: str
    attributes: str
    exception: str

"""Sidecar handlers for crawler job start/cancel/status.

Author: Xiaoman
Created: 2026-07-16
"""

from __future__ import annotations

import logging
from typing import Any

from crawler import get_default_service
from shared.logging import bind_log_context

logger = logging.getLogger("opendesk.sidecar.crawler")


def handle_crawler_job_start(payload: dict[str, Any] | None, *, trace_id: str) -> dict[str, Any]:
    """Start a crawl job.

    Contract: ``contracts/schema/v1/crawler/sidecar/job_start.*.schema.json``

    Args:
        payload: Sidecar job_start request body.
        trace_id: Trace id from Rust.

    Returns:
        Job start response with ``job_id``.
    """
    body = dict(payload or {})
    if trace_id and "trace_id" not in body:
        body["trace_id"] = trace_id
    with bind_log_context(trace_id=trace_id or body.get("trace_id", ""), feature="crawler"):
        logger.info("handle_crawler_job_start", extra={"event": "crawler.job_start"})
        try:
            return get_default_service().start(body)
        except ValueError as exc:
            logger.warning(
                "crawler job start rejected: %s",
                exc,
                extra={"event": "crawler.job_start_rejected"},
            )
            return {"ok": False, "job_id": "", "trace_id": trace_id}


def handle_crawler_job_cancel(payload: dict[str, Any] | None, *, trace_id: str) -> dict[str, Any]:
    """Cancel a crawl job.

    Contract: ``contracts/schema/v1/crawler/sidecar/job_cancel.*.schema.json``

    Args:
        payload: Must include ``job_id``.
        trace_id: Trace id from Rust.

    Returns:
        Cancel response.
    """
    body = dict(payload or {})
    if trace_id and "trace_id" not in body:
        body["trace_id"] = trace_id
    with bind_log_context(trace_id=trace_id or body.get("trace_id", ""), feature="crawler"):
        logger.info("handle_crawler_job_cancel", extra={"event": "crawler.job_cancel"})
        try:
            return get_default_service().cancel(body)
        except (ValueError, KeyError) as exc:
            logger.warning(
                "crawler job cancel rejected: %s",
                exc,
                extra={"event": "crawler.job_cancel_rejected"},
            )
            return {
                "ok": False,
                "job_id": str(body.get("job_id") or ""),
                "trace_id": trace_id,
            }


def handle_crawler_job_status(payload: dict[str, Any] | None, *, trace_id: str) -> dict[str, Any]:
    """Query crawl job status.

    Contract: ``contracts/schema/v1/crawler/sidecar/job_status.*.schema.json``

    Args:
        payload: Must include ``job_id``.
        trace_id: Trace id from Rust.

    Returns:
        Status response.
    """
    body = dict(payload or {})
    if trace_id and "trace_id" not in body:
        body["trace_id"] = trace_id
    with bind_log_context(trace_id=trace_id or body.get("trace_id", ""), feature="crawler"):
        logger.info("handle_crawler_job_status", extra={"event": "crawler.job_status"})
        try:
            return get_default_service().status(body)
        except (ValueError, KeyError) as exc:
            logger.warning(
                "crawler job status rejected: %s",
                exc,
                extra={"event": "crawler.job_status_rejected"},
            )
            return {
                "ok": False,
                "job_id": str(body.get("job_id") or ""),
                "platform": "",
                "status": "unknown",
                "trace_id": trace_id,
            }

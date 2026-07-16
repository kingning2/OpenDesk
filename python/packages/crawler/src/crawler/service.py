"""Crawl job service — start / cancel / status (in-memory skeleton).

Author: Xiaoman
Created: 2026-07-16
"""

from __future__ import annotations

import logging
import threading
import uuid
from dataclasses import dataclass, field
from typing import Any

from crawler.events import InMemoryCrawlEventEmitter
from crawler.platforms.registry import get_adapter
from crawler.ports import JobConfig

logger = logging.getLogger("opendesk.crawler.service")


def _split_csv(value: str | None) -> list[str]:
    """Split a comma-separated string into trimmed non-empty parts.

    Args:
        value: Raw CSV string or None.

    Returns:
        List of tokens.
    """
    if not value:
        return []
    return [part.strip() for part in value.split(",") if part.strip()]


def parse_job_config(payload: dict[str, Any]) -> JobConfig:
    """Build ``JobConfig`` from a sidecar/IPC request payload.

    Args:
        payload: Contract-shaped request dict.

    Returns:
        Normalized ``JobConfig``.

    Raises:
        ValueError: When required fields are missing or invalid.
    """
    platform = str(payload.get("platform") or "").strip()
    keywords = _split_csv(str(payload.get("keywords") or ""))
    if not platform:
        raise ValueError("platform is required")
    if not keywords:
        raise ValueError("keywords is required")
    return JobConfig(
        platform=platform,
        keywords=keywords,
        rate_limit_ms=int(payload.get("rate_limit_ms") or 0),
        max_total=int(payload.get("max_total") or 100),
        year=int(payload.get("year") or 2025),
        min_year_video_count=int(payload.get("min_year_video_count") or 10),
        exclude_countries=_split_csv(str(payload.get("exclude_countries") or "")),
        batch_id=str(payload.get("batch_id") or ""),
    )


@dataclass
class JobState:
    """Runtime state for one crawl job.

    Attributes:
        job_id: Unique job id.
        platform: Platform id.
        status: ``queued`` | ``running`` | ``completed`` | ``failed`` | ``cancelled``.
        config: Job configuration.
        cancel_requested: Cancellation flag.
        stop_reason: Terminal stop reason when finished.
        scanned_count: Channels scanned.
        accepted_count: Channels accepted.
        error: Failure message if any.
        emitter: In-memory event buffer for this job.
    """

    job_id: str
    platform: str
    status: str
    config: JobConfig
    cancel_requested: bool = False
    stop_reason: str = ""
    scanned_count: int = 0
    accepted_count: int = 0
    error: str = ""
    emitter: InMemoryCrawlEventEmitter = field(default_factory=InMemoryCrawlEventEmitter)


class CrawlJobService:
    """In-memory crawl job orchestrator (no persistence).

    Runs adapters on background threads so ``start`` returns ``job_id`` immediately.
    """

    def __init__(self) -> None:
        self._lock = threading.Lock()
        self._jobs: dict[str, JobState] = {}

    def start(self, payload: dict[str, Any]) -> dict[str, Any]:
        """Start a crawl job.

        Args:
            payload: Sidecar ``job_start`` request fields.

        Returns:
            ``{ok, job_id, trace_id?}``.
        """
        config = parse_job_config(payload)
        # Resolve adapter early to fail fast on unknown platform.
        get_adapter(config.platform)
        job_id = str(uuid.uuid4())
        state = JobState(
            job_id=job_id,
            platform=config.platform,
            status="queued",
            config=config,
        )
        with self._lock:
            self._jobs[job_id] = state

        thread = threading.Thread(
            target=self._run_job,
            args=(job_id,),
            name=f"crawler-{job_id[:8]}",
            daemon=True,
        )
        thread.start()

        result: dict[str, Any] = {"ok": True, "job_id": job_id}
        if payload.get("trace_id"):
            result["trace_id"] = str(payload["trace_id"])
        return result

    def cancel(self, payload: dict[str, Any]) -> dict[str, Any]:
        """Request cancellation of a running job.

        Args:
            payload: Must include ``job_id``.

        Returns:
            ``{ok, job_id, status, trace_id?}``.
        """
        job_id = str(payload.get("job_id") or "").strip()
        if not job_id:
            raise ValueError("job_id is required")
        with self._lock:
            state = self._jobs.get(job_id)
            if state is None:
                raise KeyError(f"unknown job_id={job_id}")
            state.cancel_requested = True
            if state.status in {"queued", "running"}:
                state.status = "cancelled"
        result: dict[str, Any] = {"ok": True, "job_id": job_id}
        if payload.get("trace_id"):
            result["trace_id"] = str(payload["trace_id"])
        return result

    def status(self, payload: dict[str, Any]) -> dict[str, Any]:
        """Query job status.

        Args:
            payload: Must include ``job_id``.

        Returns:
            Contract-shaped status response.
        """
        job_id = str(payload.get("job_id") or "").strip()
        if not job_id:
            raise ValueError("job_id is required")
        with self._lock:
            state = self._jobs.get(job_id)
            if state is None:
                raise KeyError(f"unknown job_id={job_id}")
            result: dict[str, Any] = {
                "ok": True,
                "job_id": state.job_id,
                "platform": state.platform,
                "status": state.status,
                "scanned_count": state.scanned_count,
                "accepted_count": state.accepted_count,
            }
            if state.stop_reason:
                result["stop_reason"] = state.stop_reason
            if payload.get("trace_id"):
                result["trace_id"] = str(payload["trace_id"])
            return result

    def logs(self, job_id: str) -> list[dict[str, Any]]:
        """Return ordered process log events for a job (test/debug helper).

        Args:
            job_id: Job identifier.

        Returns:
            ``crawler.job.log`` payloads sorted by ``seq``.
        """
        with self._lock:
            state = self._jobs.get(job_id)
            if state is None:
                raise KeyError(f"unknown job_id={job_id}")
            return list(state.emitter.logs_for(job_id))

    def _run_job(self, job_id: str) -> None:
        """Background worker for one job.

        Args:
            job_id: Job to execute.
        """
        with self._lock:
            state = self._jobs[job_id]
            state.status = "running"
            config = state.config
            emitter = state.emitter

        try:
            adapter = get_adapter(config.platform)
            summary = adapter.run(
                job_id,
                config,
                emitter,
                should_cancel=lambda: self._is_cancel_requested(job_id),
            )
            with self._lock:
                state = self._jobs[job_id]
                state.scanned_count = int(summary.get("scanned_count") or 0)
                state.accepted_count = int(summary.get("accepted_count") or 0)
                state.stop_reason = str(summary.get("stop_reason") or "")
                if state.stop_reason == "cancelled" or state.cancel_requested:
                    state.status = "cancelled"
                else:
                    state.status = "completed"
        except Exception as exc:
            logger.exception(
                "crawl job failed",
                extra={"event": "crawler.job_failed", "feature": "crawler", "task_id": job_id},
            )
            with self._lock:
                state = self._jobs[job_id]
                state.status = "failed"
                state.error = str(exc)
                state.stop_reason = "failed"
            state.emitter.emit(
                "crawler.job.failed",
                {
                    "event_id": str(uuid.uuid4()),
                    "occurred_at": "",
                    "job_id": job_id,
                    "platform": config.platform,
                    "error_code": "adapter_error",
                    "message": str(exc),
                },
            )

    def _is_cancel_requested(self, job_id: str) -> bool:
        """Check whether cancellation was requested.

        Args:
            job_id: Job identifier.

        Returns:
            True when cancel was requested.
        """
        with self._lock:
            state = self._jobs.get(job_id)
            return bool(state and state.cancel_requested)


_DEFAULT_SERVICE: CrawlJobService | None = None
_DEFAULT_LOCK = threading.Lock()


def get_default_service() -> CrawlJobService:
    """Return process-wide default ``CrawlJobService`` singleton.

    Returns:
        Shared service instance.
    """
    global _DEFAULT_SERVICE
    with _DEFAULT_LOCK:
        if _DEFAULT_SERVICE is None:
            _DEFAULT_SERVICE = CrawlJobService()
        return _DEFAULT_SERVICE

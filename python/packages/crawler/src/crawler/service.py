"""Crawl job service — start / cancel / status (in-memory).

Author: Xiaoman
Created: 2026-07-16
"""

from __future__ import annotations

import json
import logging
import threading
import uuid
from dataclasses import dataclass, field
from typing import Any

from crawler.events import InMemoryCrawlEventEmitter
from crawler.platforms.registry import get_adapter
from crawler.ports import CrawlEventEmitter, JobConfig

logger = logging.getLogger("opendesk.crawler.service")

_STOP_MESSAGES = {
    "keywords_finished": "全部关键词已爬完",
    "max_total_reached": "已达到本次数上限，自动停止",
    "quota_exceeded": "YouTube 配额已用尽，已自动停止爬虫",
    "cancelled": "任务已取消",
    "failed": "任务失败",
}


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
        api_key=str(payload.get("api_key") or ""),
    )


@dataclass
class KeywordStat:
    """Per-keyword crawl counters."""

    keyword: str
    scanned: int = 0
    accepted: int = 0


@dataclass
class JobState:
    """Runtime state for one crawl job."""

    job_id: str
    platform: str
    status: str
    config: JobConfig
    cancel_requested: bool = False
    stop_reason: str = ""
    scanned_count: int = 0
    accepted_count: int = 0
    keyword_scanned: int = 0
    keyword_accepted: int = 0
    quota_used: int = 0
    current_keyword: str = ""
    message: str = "等待开始"
    error: str = ""
    keyword_stats: list[KeywordStat] = field(default_factory=list)
    emitter: InMemoryCrawlEventEmitter = field(default_factory=InMemoryCrawlEventEmitter)


class ProgressTrackingEmitter:
    """Wraps an emitter and mirrors progress into ``JobState`` for UI polling."""

    def __init__(
        self,
        inner: CrawlEventEmitter,
        state: JobState,
        lock: threading.Lock,
    ) -> None:
        self._inner = inner
        self._state = state
        self._lock = lock

    def emit(self, topic: str, payload: dict[str, Any]) -> None:
        """Forward event and update operational progress fields.

        Args:
            topic: Event topic.
            payload: Event payload.
        """
        self._inner.emit(topic, payload)
        with self._lock:
            if topic == "crawler.job.progress":
                keyword = str(payload.get("current_keyword") or "")
                self._state.current_keyword = keyword
                self._state.scanned_count = int(payload.get("scanned_count") or 0)
                self._state.accepted_count = int(payload.get("accepted_count") or 0)
                self._state.keyword_scanned = int(payload.get("keyword_scanned") or 0)
                self._state.keyword_accepted = int(payload.get("keyword_accepted") or 0)
                self._state.quota_used = int(payload.get("quota_used") or 0)
                self._upsert_keyword_stat(
                    keyword,
                    scanned=self._state.keyword_scanned,
                    accepted=self._state.keyword_accepted,
                )
                self._state.message = (
                    f"正在爬关键词「{keyword}」：本词已收录 "
                    f"{self._state.keyword_accepted} 条，扫描 "
                    f"{self._state.keyword_scanned} 条；合计收录 "
                    f"{self._state.accepted_count} 条"
                )
            elif topic == "crawler.job.started":
                self._state.message = "任务已启动，准备爬取关键词"
            elif topic == "crawler.job.completed":
                reason = str(payload.get("stop_reason") or "")
                self._state.stop_reason = reason
                self._state.scanned_count = int(
                    payload.get("scanned_count") or self._state.scanned_count
                )
                self._state.accepted_count = int(
                    payload.get("accepted_count") or self._state.accepted_count
                )
                self._state.quota_used = int(payload.get("quota_used") or self._state.quota_used)
                self._state.message = _STOP_MESSAGES.get(reason, f"已结束（{reason}）")
                if reason == "quota_exceeded":
                    self._state.message = "YouTube 配额已用尽，已自动停止爬虫"
            elif topic == "crawler.job.failed":
                self._state.error = str(payload.get("message") or "任务失败")
                code = str(payload.get("error_code") or "")
                if code == "cancelled":
                    self._state.message = _STOP_MESSAGES["cancelled"]
                else:
                    self._state.message = f"失败：{self._state.error}"

    def _upsert_keyword_stat(self, keyword: str, *, scanned: int, accepted: int) -> None:
        if not keyword:
            return
        for row in self._state.keyword_stats:
            if row.keyword == keyword:
                row.scanned = scanned
                row.accepted = accepted
                return
        self._state.keyword_stats.append(
            KeywordStat(keyword=keyword, scanned=scanned, accepted=accepted)
        )


class CrawlJobService:
    """In-memory crawl job orchestrator (no persistence)."""

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
        get_adapter(config.platform, api_key=config.api_key)
        job_id = str(uuid.uuid4())
        state = JobState(
            job_id=job_id,
            platform=config.platform,
            status="queued",
            config=config,
            message="排队中",
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
            ``{ok, job_id, trace_id?}``.
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
                state.stop_reason = "cancelled"
                state.message = _STOP_MESSAGES["cancelled"]
        result: dict[str, Any] = {"ok": True, "job_id": job_id}
        if payload.get("trace_id"):
            result["trace_id"] = str(payload["trace_id"])
        return result

    def status(self, payload: dict[str, Any]) -> dict[str, Any]:
        """Query operational job progress for the desktop UI.

        Args:
            payload: Must include ``job_id``.

        Returns:
            Contract-shaped status response with keyword/progress fields.
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
                "message": state.message,
                "scanned_count": state.scanned_count,
                "accepted_count": state.accepted_count,
                "keyword_scanned": state.keyword_scanned,
                "keyword_accepted": state.keyword_accepted,
                "quota_used": state.quota_used,
                "keyword_stats_json": json.dumps(
                    [
                        {
                            "keyword": row.keyword,
                            "scanned": row.scanned,
                            "accepted": row.accepted,
                        }
                        for row in state.keyword_stats
                    ],
                    ensure_ascii=False,
                ),
            }
            if state.current_keyword:
                result["current_keyword"] = state.current_keyword
            if state.stop_reason:
                result["stop_reason"] = state.stop_reason
            if state.error:
                result["error_message"] = state.error
            if payload.get("trace_id"):
                result["trace_id"] = str(payload["trace_id"])
            return result

    def logs(self, payload: dict[str, Any]) -> dict[str, Any]:
        """Return process logs for a job as JSON string (debug).

        Args:
            payload: Must include ``job_id``.

        Returns:
            ``{ok, job_id, logs_json, trace_id?}``.
        """
        job_id = str(payload.get("job_id") or "").strip()
        if not job_id:
            raise ValueError("job_id is required")
        with self._lock:
            state = self._jobs.get(job_id)
            if state is None:
                raise KeyError(f"unknown job_id={job_id}")
            rows = list(state.emitter.logs_for(job_id))
        result: dict[str, Any] = {
            "ok": True,
            "job_id": job_id,
            "logs_json": json.dumps(rows, ensure_ascii=False),
        }
        if payload.get("trace_id"):
            result["trace_id"] = str(payload["trace_id"])
        return result

    def _run_job(self, job_id: str) -> None:
        """Background worker for one job.

        Args:
            job_id: Job to execute.
        """
        with self._lock:
            state = self._jobs[job_id]
            state.status = "running"
            state.message = "正在启动爬虫…"
            config = state.config
            tracker = ProgressTrackingEmitter(state.emitter, state, self._lock)

        try:
            adapter = get_adapter(config.platform, api_key=config.api_key)
            summary = adapter.run(
                job_id,
                config,
                tracker,
                should_cancel=lambda: self._is_cancel_requested(job_id),
            )
            with self._lock:
                state = self._jobs[job_id]
                state.scanned_count = int(summary.get("scanned_count") or 0)
                state.accepted_count = int(summary.get("accepted_count") or 0)
                state.quota_used = int(summary.get("quota_used") or state.quota_used)
                state.stop_reason = str(summary.get("stop_reason") or "")
                if state.stop_reason == "cancelled" or state.cancel_requested:
                    state.status = "cancelled"
                    state.message = _STOP_MESSAGES["cancelled"]
                elif state.stop_reason == "quota_exceeded":
                    # Treat quota stop as completed-with-reason so UI can show auto-stop.
                    state.status = "completed"
                    state.message = _STOP_MESSAGES["quota_exceeded"]
                else:
                    state.status = "completed"
                    state.message = _STOP_MESSAGES.get(
                        state.stop_reason,
                        f"已结束（{state.stop_reason}）",
                    )
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
                state.message = f"失败：{exc}"
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

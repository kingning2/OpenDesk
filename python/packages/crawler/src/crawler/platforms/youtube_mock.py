"""YouTube mock platform adapter (skeleton - no real HTTP).

Author: Xiaoman
Created: 2026-07-16
"""

from __future__ import annotations

import logging
import time
import uuid
from collections.abc import Callable
from datetime import UTC, datetime
from typing import Any

from crawler.ports import ChannelHit, CrawlEventEmitter, JobConfig
from crawler.quota import calculate_expected_quota, get_endpoint_cost
from shared.logging import bind_log_context

logger = logging.getLogger("opendesk.crawler.youtube")

ShouldCancel = Callable[[], bool]


def _now_iso() -> str:
    """Return UTC timestamp in ISO-8601 form.

    Returns:
        ISO-8601 datetime string.
    """
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def _new_event_id() -> str:
    """Allocate a unique event id.

    Returns:
        UUID string.
    """
    return str(uuid.uuid4())


class YoutubeMockAdapter:
    """Deterministic YouTube crawl mock that emits process logs and progress.

    Attributes:
        platform: Platform id ``youtube``.
    """

    platform = "youtube"

    def run(
        self,
        job_id: str,
        config: JobConfig,
        emitter: CrawlEventEmitter,
        *,
        should_cancel: ShouldCancel,
    ) -> dict[str, Any]:
        """Simulate keyword → search → channel batch → filter for each keyword.

        Args:
            job_id: Job identifier.
            config: Normalized job config.
            emitter: Domain event emitter.
            should_cancel: Cancellation probe between steps.

        Returns:
            Summary with ``stop_reason``, counts, and ``quota_used``.
        """
        seq = 0
        scanned = 0
        accepted = 0
        search_pages = 0
        channel_list_calls = 0
        started = time.monotonic()
        hits: list[ChannelHit] = []

        def emit_log(
            phase: str,
            message: str,
            *,
            level: str = "INFO",
            keyword: str = "",
            detail: str = "",
        ) -> None:
            nonlocal seq
            seq += 1
            payload: dict[str, Any] = {
                "event_id": _new_event_id(),
                "occurred_at": _now_iso(),
                "job_id": job_id,
                "platform": self.platform,
                "seq": seq,
                "phase": phase,
                "level": level,
                "message": message,
            }
            if keyword:
                payload["keyword"] = keyword
            if detail:
                payload["detail"] = detail
            emitter.emit("crawler.job.log", payload)
            logger.info(
                message,
                extra={
                    "event": f"crawler.{phase}",
                    "feature": "crawler",
                    "task_id": job_id,
                    "phase": phase,
                    "seq": seq,
                    "keyword": keyword or None,
                },
            )

        def emit_progress(
            keyword: str,
            *,
            keyword_scanned: int,
            keyword_accepted: int,
        ) -> None:
            emitter.emit(
                "crawler.job.progress",
                {
                    "event_id": _new_event_id(),
                    "occurred_at": _now_iso(),
                    "job_id": job_id,
                    "platform": self.platform,
                    "current_keyword": keyword,
                    "scanned_count": scanned,
                    "accepted_count": accepted,
                    "keyword_scanned": keyword_scanned,
                    "keyword_accepted": keyword_accepted,
                    "quota_used": calculate_expected_quota(
                        search_pages=search_pages,
                        channel_list_calls=channel_list_calls,
                    ),
                    "search_pages": search_pages,
                },
            )

        with bind_log_context(task_id=job_id, feature="crawler"):
            emit_log("job_started", f"mock youtube job started keywords={len(config.keywords)}")
            emitter.emit(
                "crawler.job.started",
                {
                    "event_id": _new_event_id(),
                    "occurred_at": _now_iso(),
                    "job_id": job_id,
                    "platform": self.platform,
                    "keywords": ",".join(config.keywords),
                },
            )

            stop_reason = "keywords_finished"
            for keyword in config.keywords:
                if should_cancel():
                    stop_reason = "cancelled"
                    emit_log("job_failed", "cancelled by client", level="WARNING", keyword=keyword)
                    break

                emit_log("keyword_begin", f"begin keyword={keyword}", keyword=keyword)
                if config.rate_limit_ms > 0:
                    time.sleep(config.rate_limit_ms / 1000.0)

                # One mock search page per keyword.
                search_pages += 1
                emit_log(
                    "search_page",
                    f"search.list page=1 cost={get_endpoint_cost('/search')}",
                    keyword=keyword,
                    detail=f'{{"page":1,"cost":{get_endpoint_cost("/search")}}}',
                )

                # Mock 3 candidate channels per keyword.
                candidates = [
                    ChannelHit(
                        platform=self.platform,
                        channel_id=f"UC_mock_{keyword}_{idx}",
                        title=f"{keyword} channel {idx}",
                        country="US" if idx != 2 else "CN",
                        subscriber_count=1000 * (idx + 1),
                        email=f"contact{idx}@{keyword}.example" if idx == 1 else "",
                        description=f"mock channel for {keyword}",
                        custom_url=f"@{keyword}{idx}",
                    )
                    for idx in range(3)
                ]
                channel_list_calls += 1
                scanned += len(candidates)
                emit_log(
                    "channel_batch",
                    f"channels.list batch size={len(candidates)}",
                    keyword=keyword,
                )

                kept = 0
                for hit in candidates:
                    if hit.country.upper() in {c.upper() for c in config.exclude_countries}:
                        emit_log(
                            "filter",
                            f"exclude channel={hit.channel_id} country={hit.country}",
                            keyword=keyword,
                            level="DEBUG",
                        )
                        continue
                    # Mock activity: idx 0 has low activity → drop when min is high.
                    year_videos = 20 if hit.subscriber_count >= 2000 else 5
                    if year_videos < config.min_year_video_count:
                        emit_log(
                            "filter",
                            f"drop inactive channel={hit.channel_id} year_videos={year_videos}",
                            keyword=keyword,
                            level="DEBUG",
                        )
                        continue
                    hits.append(hit)
                    accepted += 1
                    kept += 1
                    if accepted >= config.max_total:
                        break

                emit_log(
                    "filter",
                    f"accepted={kept} scanned_batch={len(candidates)}",
                    keyword=keyword,
                )
                emit_progress(
                    keyword,
                    keyword_scanned=len(candidates),
                    keyword_accepted=kept,
                )
                emit_log("keyword_done", f"keyword done accepted_total={accepted}", keyword=keyword)

                if accepted >= config.max_total:
                    stop_reason = "max_total_reached"
                    break

            duration_ms = int((time.monotonic() - started) * 1000)
            quota_used = calculate_expected_quota(
                search_pages=search_pages,
                channel_list_calls=channel_list_calls,
            )

            if stop_reason == "cancelled":
                emitter.emit(
                    "crawler.job.failed",
                    {
                        "event_id": _new_event_id(),
                        "occurred_at": _now_iso(),
                        "job_id": job_id,
                        "platform": self.platform,
                        "error_code": "cancelled",
                        "message": "job cancelled",
                    },
                )
            else:
                emit_log(
                    "job_completed",
                    f"completed stop_reason={stop_reason} accepted={accepted}",
                )
                emitter.emit(
                    "crawler.job.completed",
                    {
                        "event_id": _new_event_id(),
                        "occurred_at": _now_iso(),
                        "job_id": job_id,
                        "platform": self.platform,
                        "stop_reason": stop_reason,
                        "scanned_count": scanned,
                        "accepted_count": accepted,
                        "quota_used": quota_used,
                        "duration_ms": duration_ms,
                    },
                )

            return {
                "stop_reason": stop_reason,
                "scanned_count": scanned,
                "accepted_count": accepted,
                "quota_used": quota_used,
                "duration_ms": duration_ms,
                "hits": hits,
            }

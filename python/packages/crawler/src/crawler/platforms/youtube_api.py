"""YouTube Data API v3 crawl adapter.

Author: Xiaoman
Created: 2026-07-16
"""

from __future__ import annotations

import json
import logging
import time
import urllib.error
import urllib.parse
import urllib.request
from collections.abc import Callable
from datetime import UTC, datetime
from typing import Any

from crawler.helpers import extract_email, new_event_id, now_iso
from crawler.ports import ChannelHit, CrawlEventEmitter, JobConfig
from crawler.quota import calculate_expected_quota, get_endpoint_cost
from shared.logging import bind_log_context

logger = logging.getLogger("opendesk.crawler.youtube_api")

ShouldCancel = Callable[[], bool]
_API_BASE = "https://www.googleapis.com/youtube/v3"
_USER_AGENT = "OpenDeskCrawler/0.1"


class YoutubeQuotaExceededError(Exception):
    """Raised when YouTube returns a quotaExceeded error."""


class YoutubeApiAdapter:
    """Live YouTube Data API crawler (keywords → channels).

    Attributes:
        platform: Platform id ``youtube``.
    """

    platform = "youtube"

    def __init__(self, api_key: str) -> None:
        """Create adapter with a caller-supplied API key.

        Args:
            api_key: YouTube Data API key from the desktop UI (via Rust).

        Raises:
            ValueError: When ``api_key`` is empty.
        """
        key = (api_key or "").strip()
        if not key:
            raise ValueError("api_key is required for YoutubeApiAdapter")
        self._api_key = key

    def run(
        self,
        job_id: str,
        config: JobConfig,
        emitter: CrawlEventEmitter,
        *,
        should_cancel: ShouldCancel,
    ) -> dict[str, Any]:
        """Crawl channels for each keyword using YouTube Data API.

        Args:
            job_id: Job identifier.
            config: Normalized job config (must not log ``api_key``).
            emitter: Domain event emitter.
            should_cancel: Cancellation probe.

        Returns:
            Summary with stop_reason, counts, and quota_used.
        """
        seq = 0
        scanned = 0
        accepted = 0
        search_pages = 0
        channel_list_calls = 0
        playlist_item_pages = 0
        started = time.monotonic()
        exclude = {c.upper() for c in config.exclude_countries}

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
                "event_id": new_event_id(),
                "occurred_at": now_iso(),
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

        def emit_progress(keyword: str) -> None:
            emitter.emit(
                "crawler.job.progress",
                {
                    "event_id": new_event_id(),
                    "occurred_at": now_iso(),
                    "job_id": job_id,
                    "platform": self.platform,
                    "current_keyword": keyword,
                    "scanned_count": scanned,
                    "accepted_count": accepted,
                    "quota_used": calculate_expected_quota(
                        search_pages=search_pages,
                        channel_list_calls=channel_list_calls,
                        playlist_item_pages=playlist_item_pages,
                    ),
                    "search_pages": search_pages,
                },
            )

        def sleep_rate() -> None:
            if config.rate_limit_ms > 0:
                time.sleep(config.rate_limit_ms / 1000.0)

        with bind_log_context(task_id=job_id, feature="crawler"):
            emit_log(
                "job_started",
                f"youtube api job started keywords={len(config.keywords)}",
            )
            emitter.emit(
                "crawler.job.started",
                {
                    "event_id": new_event_id(),
                    "occurred_at": now_iso(),
                    "job_id": job_id,
                    "platform": self.platform,
                    "keywords": ",".join(config.keywords),
                },
            )

            stop_reason = "keywords_finished"
            try:
                for keyword in config.keywords:
                    if should_cancel():
                        stop_reason = "cancelled"
                        emit_log(
                            "job_failed",
                            "cancelled by client",
                            level="WARNING",
                            keyword=keyword,
                        )
                        break
                    if accepted >= config.max_total:
                        stop_reason = "max_total_reached"
                        break

                    emit_log("keyword_begin", f"begin keyword={keyword}", keyword=keyword)
                    page_token: str | None = None

                    while accepted < config.max_total:
                        if should_cancel():
                            stop_reason = "cancelled"
                            break

                        sleep_rate()
                        search_pages += 1
                        search_params: dict[str, str] = {
                            "part": "snippet",
                            "q": keyword,
                            "type": "channel",
                            "maxResults": "50",
                        }
                        if page_token:
                            search_params["pageToken"] = page_token
                        search_body = self._get("/search", search_params)
                        emit_log(
                            "search_page",
                            f"search.list cost={get_endpoint_cost('/search')}",
                            keyword=keyword,
                        )

                        items = search_body.get("items") or []
                        channel_ids = [
                            str(
                                item.get("snippet", {}).get("channelId")
                                or item.get("id", {}).get("channelId")
                                or ""
                            )
                            for item in items
                        ]
                        channel_ids = [cid for cid in channel_ids if cid]
                        if not channel_ids:
                            page_token = search_body.get("nextPageToken")
                            if not page_token:
                                break
                            continue

                        sleep_rate()
                        channel_list_calls += 1
                        channels_body = self._get(
                            "/channels",
                            {
                                "part": "snippet,statistics,contentDetails",
                                "id": ",".join(channel_ids),
                            },
                        )
                        channel_items = channels_body.get("items") or []
                        scanned += len(channel_items)
                        emit_log(
                            "channel_batch",
                            f"channels.list size={len(channel_items)}",
                            keyword=keyword,
                        )

                        for channel in channel_items:
                            if accepted >= config.max_total:
                                stop_reason = "max_total_reached"
                                break
                            if should_cancel():
                                stop_reason = "cancelled"
                                break

                            hit = self._to_hit(channel)
                            country = hit.country.upper()
                            if country and country in exclude:
                                emit_log(
                                    "filter",
                                    f"exclude channel={hit.channel_id} country={hit.country}",
                                    keyword=keyword,
                                    level="DEBUG",
                                )
                                continue

                            uploads_id = (
                                channel.get("contentDetails", {})
                                .get("relatedPlaylists", {})
                                .get("uploads")
                            )
                            year_videos = 0
                            if uploads_id:
                                sleep_rate()
                                year_videos, pages = self._count_year_videos(
                                    str(uploads_id),
                                    config.year,
                                )
                                playlist_item_pages += pages

                            if year_videos < config.min_year_video_count:
                                emit_log(
                                    "filter",
                                    (
                                        f"drop inactive channel={hit.channel_id} "
                                        f"year_videos={year_videos}"
                                    ),
                                    keyword=keyword,
                                    level="DEBUG",
                                )
                                continue

                            accepted += 1
                            emit_log(
                                "filter",
                                f"accept channel={hit.channel_id} title={hit.title}",
                                keyword=keyword,
                            )

                        quota_used = calculate_expected_quota(
                            search_pages=search_pages,
                            channel_list_calls=channel_list_calls,
                            playlist_item_pages=playlist_item_pages,
                        )
                        emit_log(
                            "quota",
                            f"quota_used={quota_used}",
                            keyword=keyword,
                        )
                        emit_progress(keyword)

                        page_token = search_body.get("nextPageToken")
                        if not page_token:
                            break
                        if stop_reason in {"cancelled", "max_total_reached"}:
                            break

                    if stop_reason == "cancelled":
                        break
                    emit_log(
                        "keyword_done",
                        f"keyword done accepted_total={accepted}",
                        keyword=keyword,
                    )
                    if stop_reason == "max_total_reached":
                        break

            except YoutubeQuotaExceededError:
                stop_reason = "quota_exceeded"
                emit_log("quota", "YouTube quotaExceeded — stopping", level="WARNING")

            duration_ms = int((time.monotonic() - started) * 1000)
            quota_used = calculate_expected_quota(
                search_pages=search_pages,
                channel_list_calls=channel_list_calls,
                playlist_item_pages=playlist_item_pages,
            )

            if stop_reason == "cancelled":
                emitter.emit(
                    "crawler.job.failed",
                    {
                        "event_id": new_event_id(),
                        "occurred_at": now_iso(),
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
                        "event_id": new_event_id(),
                        "occurred_at": now_iso(),
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
            }

    def _get(self, path: str, params: dict[str, str]) -> dict[str, Any]:
        """GET a YouTube Data API resource.

        Args:
            path: API path starting with ``/``.
            params: Query parameters without the API key.

        Returns:
            Parsed JSON object.

        Raises:
            YoutubeQuotaExceededError: When quota is exceeded.
            RuntimeError: On other HTTP/API failures.
        """
        query = dict(params)
        query["key"] = self._api_key
        url = f"{_API_BASE}{path}?{urllib.parse.urlencode(query)}"
        request = urllib.request.Request(
            url,
            headers={"Accept": "application/json", "User-Agent": _USER_AGENT},
            method="GET",
        )
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                raw = response.read().decode("utf-8")
                return json.loads(raw) if raw else {}
        except urllib.error.HTTPError as exc:
            body = exc.read().decode("utf-8", errors="replace")
            if self._is_quota_exceeded(body):
                raise YoutubeQuotaExceededError(body) from exc
            raise RuntimeError(f"YouTube API HTTP {exc.code}: {body[:500]}") from exc
        except urllib.error.URLError as exc:
            raise RuntimeError(f"YouTube API network error: {exc.reason}") from exc

    def _count_year_videos(self, uploads_playlist_id: str, year: int) -> tuple[int, int]:
        """Count videos published in ``year`` from an uploads playlist.

        Args:
            uploads_playlist_id: Channel uploads playlist id.
            year: Target calendar year.

        Returns:
            Tuple of ``(count, playlist_item_pages_used)``.
        """
        count = 0
        pages = 0
        page_token: str | None = None
        year_start = datetime(year, 1, 1, tzinfo=UTC).timestamp()
        year_end = datetime(year, 12, 31, 23, 59, 59, tzinfo=UTC).timestamp()

        while True:
            pages += 1
            params: dict[str, str] = {
                "part": "contentDetails",
                "playlistId": uploads_playlist_id,
                "maxResults": "50",
            }
            if page_token:
                params["pageToken"] = page_token
            try:
                body = self._get("/playlistItems", params)
            except RuntimeError as exc:
                if "playlistNotFound" in str(exc):
                    return 0, pages
                raise

            for item in body.get("items") or []:
                published = item.get("contentDetails", {}).get("videoPublishedAt")
                if not published:
                    continue
                try:
                    normalized = str(published).replace("Z", "+00:00")
                    ts = datetime.fromisoformat(normalized).timestamp()
                except ValueError:
                    continue
                if ts < year_start:
                    return count, pages
                if year_start <= ts <= year_end:
                    count += 1

            page_token = body.get("nextPageToken")
            if not page_token:
                break
        return count, pages

    def _to_hit(self, channel: dict[str, Any]) -> ChannelHit:
        """Map a channels.list item to ``ChannelHit``.

        Args:
            channel: YouTube channel resource.

        Returns:
            Normalized channel hit.
        """
        snippet = channel.get("snippet") or {}
        statistics = channel.get("statistics") or {}
        description = str(snippet.get("description") or "")
        subs_raw = statistics.get("subscriberCount")
        try:
            subscriber_count = int(subs_raw) if subs_raw is not None else 0
        except (TypeError, ValueError):
            subscriber_count = 0
        return ChannelHit(
            platform=self.platform,
            channel_id=str(channel.get("id") or ""),
            title=str(snippet.get("title") or ""),
            country=str(snippet.get("country") or ""),
            subscriber_count=subscriber_count,
            email=extract_email(description),
            description=description,
            custom_url=str(snippet.get("customUrl") or ""),
        )

    @staticmethod
    def _is_quota_exceeded(body: str) -> bool:
        """Detect YouTube quotaExceeded errors in an error payload.

        Args:
            body: Raw HTTP error body.

        Returns:
            True when the body indicates quota exhaustion.
        """
        lowered = body.lower()
        return "quotaexceeded" in lowered or "quota exceeded" in lowered

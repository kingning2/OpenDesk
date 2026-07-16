"""Crawl ports and shared types.

Author: Xiaoman
Created: 2026-07-16
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Protocol


@dataclass
class JobConfig:
    """Normalized crawl job configuration.

    Args:
        platform: Target platform id (v1: ``youtube``).
        keywords: Search keywords (already split).
        rate_limit_ms: Delay between mock/API steps.
        max_total: Max accepted channels.
        year: Activity filter year.
        min_year_video_count: Min videos in ``year``.
        exclude_countries: Country codes/names to exclude.
        batch_id: Optional batch identifier.
    """

    platform: str
    keywords: list[str]
    rate_limit_ms: int = 0
    max_total: int = 100
    year: int = 2025
    min_year_video_count: int = 10
    exclude_countries: list[str] = field(default_factory=list)
    batch_id: str = ""


@dataclass
class ChannelHit:
    """One accepted channel from a platform adapter."""

    platform: str
    channel_id: str
    title: str
    country: str = ""
    subscriber_count: int = 0
    email: str = ""
    description: str = ""
    custom_url: str = ""


class CrawlEventEmitter(Protocol):
    """Emits domain events for a crawl job (no persistence)."""

    def emit(self, topic: str, payload: dict[str, Any]) -> None:
        """Publish one domain event.

        Args:
            topic: Event topic, e.g. ``crawler.job.log``.
            payload: Contract-shaped event payload.
        """


class PlatformAdapter(Protocol):
    """Platform-specific crawl adapter (YouTube, future platforms)."""

    platform: str

    def run(
        self,
        job_id: str,
        config: JobConfig,
        emitter: CrawlEventEmitter,
        *,
        should_cancel: Any,
    ) -> dict[str, Any]:
        """Execute crawl for one job.

        Args:
            job_id: Job identifier.
            config: Normalized config.
            emitter: Domain event emitter.
            should_cancel: Callable ``() -> bool`` checked between steps.

        Returns:
            Summary dict with stop_reason, scanned_count, accepted_count, quota_used.
        """

"""In-memory domain event emitter for crawl jobs.

Author: Xiaoman
Created: 2026-07-16
"""

from __future__ import annotations

from typing import Any


class InMemoryCrawlEventEmitter:
    """Collects crawl domain events for tests and skeleton wiring.

    Attributes:
        events: Ordered list of ``(topic, payload)`` tuples.
    """

    def __init__(self) -> None:
        self.events: list[tuple[str, dict[str, Any]]] = []

    def emit(self, topic: str, payload: dict[str, Any]) -> None:
        """Append one event.

        Args:
            topic: Event topic.
            payload: Event payload.
        """
        self.events.append((topic, payload))

    def logs_for(self, job_id: str) -> list[dict[str, Any]]:
        """Return ``crawler.job.log`` payloads for a job, ordered by seq.

        Args:
            job_id: Job id to filter.

        Returns:
            Log event payloads.
        """
        rows = [
            payload
            for topic, payload in self.events
            if topic == "crawler.job.log" and payload.get("job_id") == job_id
        ]
        return sorted(rows, key=lambda row: int(row.get("seq", 0)))

"""Crawler package tests — mock phases, quota, process logs.

Author: Xiaoman
Created: 2026-07-16
"""

from __future__ import annotations

import time

from crawler.events import InMemoryCrawlEventEmitter
from crawler.platforms.youtube_mock import YoutubeMockAdapter
from crawler.ports import JobConfig
from crawler.quota import calculate_expected_quota, get_endpoint_cost
from crawler.service import CrawlJobService


def test_import_crawler() -> None:
    """Package import smoke test."""
    import crawler

    assert crawler is not None


def test_quota_costs_match_kol_nest() -> None:
    """Quota units mirror YouTube / kol-nest calculator."""
    assert get_endpoint_cost("/search") == 100
    assert get_endpoint_cost("/channels") == 1
    assert calculate_expected_quota(search_pages=2, channel_list_calls=3) == 203


def test_youtube_mock_emits_ordered_process_logs() -> None:
    """Mock adapter emits monotonic job.log phases for UI process panel."""
    emitter = InMemoryCrawlEventEmitter()
    adapter = YoutubeMockAdapter()
    config = JobConfig(
        platform="youtube",
        keywords=["beauty", "tech"],
        rate_limit_ms=0,
        max_total=10,
        min_year_video_count=10,
        exclude_countries=["CN"],
    )
    summary = adapter.run("job-1", config, emitter, should_cancel=lambda: False)

    logs = emitter.logs_for("job-1")
    assert logs, "expected process logs"
    seqs = [int(row["seq"]) for row in logs]
    assert seqs == list(range(1, len(seqs) + 1))

    phases = [row["phase"] for row in logs]
    assert "job_started" in phases
    assert "keyword_begin" in phases
    assert "search_page" in phases
    assert "channel_batch" in phases
    assert "filter" in phases
    assert "quota" in phases
    assert "keyword_done" in phases
    assert "job_completed" in phases

    topics = [topic for topic, _ in emitter.events]
    assert "crawler.job.started" in topics
    assert "crawler.job.progress" in topics
    assert "crawler.job.completed" in topics
    assert summary["accepted_count"] >= 1
    assert summary["stop_reason"] == "keywords_finished"


def test_service_start_status_and_logs() -> None:
    """Service start returns job_id; status eventually completes; logs visible."""
    service = CrawlJobService()
    started = service.start(
        {
            "platform": "youtube",
            "keywords": "skincare",
            "rate_limit_ms": 0,
            "max_total": 5,
            "exclude_countries": "CN",
            "trace_id": "t-1",
        }
    )
    assert started["ok"] is True
    job_id = started["job_id"]
    assert job_id

    deadline = time.time() + 5
    status = service.status({"job_id": job_id})
    while status["status"] in {"queued", "running"} and time.time() < deadline:
        time.sleep(0.05)
        status = service.status({"job_id": job_id})

    assert status["status"] == "completed"
    assert status["platform"] == "youtube"
    logs = service.logs(job_id)
    assert any(row["phase"] == "job_completed" for row in logs)


def test_unsupported_platform_raises() -> None:
    """Unknown platform fails fast on start."""
    service = CrawlJobService()
    try:
        service.start({"platform": "tiktok", "keywords": "a"})
        raised = False
    except ValueError:
        raised = True
    assert raised

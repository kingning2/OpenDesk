"""Sidecar route table — consumed by Rust-managed HTTP server."""

from __future__ import annotations

ROUTES: dict[str, tuple[str, str]] = {}

ROUTES["/v1/agent/ping"] = ("POST", "handle_agent_ping")
ROUTES["/v1/crawler/job/start"] = ("POST", "handle_crawler_job_start")
ROUTES["/v1/crawler/job/cancel"] = ("POST", "handle_crawler_job_cancel")
ROUTES["/v1/crawler/job/status"] = ("POST", "handle_crawler_job_status")
ROUTES["/v1/crawler/job/logs"] = ("POST", "handle_crawler_job_logs")

"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerSidecarJobStartRequest(TypedDict, total=False):
    trace_id: str
    platform: str
    keywords: str
    rate_limit_ms: int
    max_total: int
    year: int
    min_year_video_count: int
    exclude_countries: str
    batch_id: str
    api_key: str

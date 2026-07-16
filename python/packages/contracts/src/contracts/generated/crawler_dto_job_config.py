"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CrawlerDtoJobConfig(TypedDict, total=False):
    platform: str
    keywords: str
    rate_limit_ms: int
    max_total: int
    year: int
    min_year_video_count: int
    exclude_countries: str
    batch_id: str

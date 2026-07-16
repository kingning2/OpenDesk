"""YouTube Data API quota cost helpers (ported from kol-nest-server).

Author: Xiaoman
Created: 2026-07-16
"""

from __future__ import annotations

# Costs mirror YouTube Data API v3 documented units / kol-nest calculator.
YOUTUBE_QUOTA_COST: dict[str, int] = {
    "/search": 100,
    "/channels": 1,
    "/playlistItems": 1,
    "/videos": 1,
}


def get_endpoint_cost(endpoint: str) -> int:
    """Return quota units for an endpoint path.

    Args:
        endpoint: API path, optionally with query string.

    Returns:
        Quota cost (defaults to 1 when unknown).
    """
    path_only = endpoint.split("?", 1)[0]
    return YOUTUBE_QUOTA_COST.get(path_only, 1)


def calculate_expected_quota(
    *,
    search_pages: int,
    channel_list_calls: int,
    playlist_item_pages: int = 0,
    video_list_calls: int = 0,
) -> int:
    """Calculate expected quota from crawl step counts.

    Args:
        search_pages: Number of ``search.list`` pages.
        channel_list_calls: Number of ``channels.list`` calls.
        playlist_item_pages: Number of ``playlistItems.list`` pages.
        video_list_calls: Number of ``videos.list`` calls.

    Returns:
        Total expected quota units.
    """
    total = 0
    total += search_pages * get_endpoint_cost("/search")
    total += channel_list_calls * get_endpoint_cost("/channels")
    total += playlist_item_pages * get_endpoint_cost("/playlistItems")
    total += video_list_calls * get_endpoint_cost("/videos")
    return total

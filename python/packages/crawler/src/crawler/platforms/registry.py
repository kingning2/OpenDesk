"""Platform adapter registry.

Author: Xiaoman
Created: 2026-07-16
"""

from __future__ import annotations

from crawler.platforms.youtube_api import YoutubeApiAdapter
from crawler.platforms.youtube_mock import YoutubeMockAdapter
from crawler.ports import PlatformAdapter


def get_adapter(platform: str, *, api_key: str = "") -> PlatformAdapter:
    """Resolve a platform adapter.

    Args:
        platform: ``youtube`` (live API) or ``youtube_mock`` (tests).
        api_key: Required for live ``youtube``; ignored for mock.

    Returns:
        Adapter instance.

    Raises:
        ValueError: When platform is unsupported or api_key missing for youtube.
    """
    if platform == "youtube_mock":
        return YoutubeMockAdapter()
    if platform == "youtube":
        return YoutubeApiAdapter(api_key=api_key)
    raise ValueError(f"unsupported platform={platform!r}; supported=[youtube, youtube_mock]")

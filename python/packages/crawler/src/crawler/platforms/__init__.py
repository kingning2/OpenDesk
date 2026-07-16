"""YouTube adapters export.

Author: Xiaoman
Created: 2026-07-16
"""

from crawler.platforms.youtube_api import YoutubeApiAdapter
from crawler.platforms.youtube_mock import YoutubeMockAdapter

__all__ = ["YoutubeApiAdapter", "YoutubeMockAdapter"]

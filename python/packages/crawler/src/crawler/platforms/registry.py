"""Platform adapter registry.

Author: Xiaoman
Created: 2026-07-16
"""

from __future__ import annotations

from crawler.platforms.youtube_mock import YoutubeMockAdapter
from crawler.ports import PlatformAdapter

_ADAPTERS: dict[str, PlatformAdapter] = {
    YoutubeMockAdapter.platform: YoutubeMockAdapter(),
}


def get_adapter(platform: str) -> PlatformAdapter:
    """Resolve a platform adapter.

    Args:
        platform: Platform id (e.g. ``youtube``).

    Returns:
        Registered adapter instance.

    Raises:
        ValueError: When platform is unsupported.
    """
    adapter = _ADAPTERS.get(platform)
    if adapter is None:
        supported = ", ".join(sorted(_ADAPTERS))
        raise ValueError(f"unsupported platform={platform!r}; supported=[{supported}]")
    return adapter

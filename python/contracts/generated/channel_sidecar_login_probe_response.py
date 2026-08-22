"""Auto-generated from contracts/schema."""

from typing import TypedDict


class ChannelSidecarLoginProbeResponse(TypedDict, total=False):
    ok: bool
    online: bool
    status: str
    final_url: str
    detail: str
    trace_id: str

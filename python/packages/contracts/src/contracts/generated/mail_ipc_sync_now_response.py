"""Auto-generated from contracts/schema."""

from typing import TypedDict


class MailIpcSyncNowResponse(TypedDict):
    job_ids_json: str
    enqueued: int

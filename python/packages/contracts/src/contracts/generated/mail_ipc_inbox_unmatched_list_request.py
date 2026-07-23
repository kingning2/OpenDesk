"""Auto-generated from contracts/schema."""

from typing import TypedDict


class MailIpcInboxUnmatchedListRequest(TypedDict, total=False):
    account_id: str
    limit: int
    offset: int

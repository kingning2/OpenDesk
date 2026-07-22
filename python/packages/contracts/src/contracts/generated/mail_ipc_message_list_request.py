"""Auto-generated from contracts/schema."""

from typing import TypedDict


class MailIpcMessageListRequest(TypedDict, total=False):
    direction: str
    account_id: str
    customer_id: str
    query: str
    limit: int
    offset: int

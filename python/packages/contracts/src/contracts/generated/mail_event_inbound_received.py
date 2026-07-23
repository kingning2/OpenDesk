"""Auto-generated from contracts/schema."""

from typing import TypedDict


class MailEventInboundReceived(TypedDict, total=False):
    event_id: str
    occurred_at: str
    message_id: str
    account_id: str
    customer_id: str
    direction: str
    subject: str
    from_address: str

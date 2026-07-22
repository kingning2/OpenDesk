"""Auto-generated from contracts/schema."""

from typing import TypedDict


class MailIpcSendRequest(TypedDict, total=False):
    customer_id: str
    account_id: str
    template_id: str
    subject: str
    body_text: str
    body_html: str

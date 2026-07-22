"""Auto-generated from contracts/schema."""

from typing import TypedDict


class MailDtoInboundRecord(TypedDict, total=False):
    customer_id: str
    from_address: str
    from_name: str
    subject: str
    body_text: str
    body_html: str
    received_at: str
    rfc_message_id: str
    in_reply_to: str

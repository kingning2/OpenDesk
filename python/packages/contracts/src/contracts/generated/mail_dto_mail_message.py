"""Auto-generated from contracts/schema."""

from typing import TypedDict


class MailDtoMailMessage(TypedDict, total=False):
    id: str
    customer_id: str
    template_id: str
    account_id: str
    status: str
    direction: str
    subject: str
    body_text: str
    body_html: str
    error_message: str
    sent_at: str
    received_at: str
    imap_uid: int
    imap_folder: str
    rfc_message_id: str
    in_reply_to: str
    references: str
    is_favorite: bool
    open_tracking_id: str
    created_at: str
    updated_at: str

"""Auto-generated from contracts/schema."""

from typing import TypedDict


class MailIpcTemplateApplyResponse(TypedDict, total=False):
    subject: str
    body_text: str
    body_html: str

"""Auto-generated from contracts/schema."""

from typing import TypedDict


class MailIpcTemplateSaveRequest(TypedDict, total=False):
    id: str
    name: str
    template_intent: str
    subject_template: str
    body_text_template: str
    body_html_template: str
    locale: str
    is_active: bool
    sort_order: int

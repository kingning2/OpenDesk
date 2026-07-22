"""Auto-generated from contracts/schema."""

from typing import TypedDict


class MailDtoMailTemplate(TypedDict, total=False):
    id: str
    name: str
    template_intent: str
    subject_template: str
    body_text_template: str
    body_html_template: str
    locale: str
    is_system: bool
    is_active: bool
    sort_order: int
    created_at: str
    updated_at: str

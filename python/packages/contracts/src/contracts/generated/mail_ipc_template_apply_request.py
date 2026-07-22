"""Auto-generated from contracts/schema."""

from typing import TypedDict


class MailIpcTemplateApplyRequest(TypedDict, total=False):
    customer_id: str
    template_id: str
    account_id: str

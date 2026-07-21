"""Auto-generated from contracts/schema."""

from typing import TypedDict


class CustomerDtoProfile(TypedDict, total=False):
    id: str
    display_name: str
    email: str
    whatsapp_phone: str
    source_channel: str
    source_meta: str
    lifecycle_status: str
    outreach_stage: str
    quoted_price: float
    quoted_currency: str
    quoted_at: str
    pricing_tier: str
    cooperation_status: str
    package_name: str
    monthly_fee: float
    contract_start: str
    contract_end: str
    notes: str
    created_at: str
    updated_at: str

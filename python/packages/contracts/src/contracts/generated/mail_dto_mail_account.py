"""Auto-generated from contracts/schema."""

from typing import TypedDict


class MailDtoMailAccount(TypedDict, total=False):
    id: str
    label: str
    from_address: str
    from_name: str
    smtp_host: str
    smtp_port: int
    use_tls: bool
    username: str
    password_ref: str
    imap_host: str
    imap_port: int
    imap_use_tls: bool
    imap_sync_enabled: bool
    created_at: str
    updated_at: str

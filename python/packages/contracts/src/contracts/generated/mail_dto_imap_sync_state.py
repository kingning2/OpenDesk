"""Auto-generated from contracts/schema."""

from typing import TypedDict


class MailDtoImapSyncState(TypedDict, total=False):
    account_id: str
    folder: str
    uidvalidity: int
    last_uid: int
    last_sync_at: str
    last_error: str
    full_synced: bool
    is_syncing: bool

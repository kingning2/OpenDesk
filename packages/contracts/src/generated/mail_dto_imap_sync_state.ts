export interface MailDtoImapSyncState {
  account_id: string;
  folder: string;
  uidvalidity?: number;
  last_uid: number;
  last_sync_at?: string;
  last_error?: string;
  full_synced: boolean;
  is_syncing?: boolean;
}

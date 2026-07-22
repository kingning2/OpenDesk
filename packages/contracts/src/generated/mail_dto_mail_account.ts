export interface MailDtoMailAccount {
  id: string;
  label: string;
  from_address: string;
  from_name?: string;
  smtp_host: string;
  smtp_port: number;
  use_tls: boolean;
  username: string;
  password_ref: string;
  imap_host?: string;
  imap_port?: number;
  imap_use_tls?: boolean;
  imap_sync_enabled: boolean;
  created_at: string;
  updated_at: string;
}

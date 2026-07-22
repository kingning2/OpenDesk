export interface MailIpcAccountSaveRequest {
  id?: string;
  label: string;
  from_address: string;
  from_name?: string;
  smtp_host: string;
  smtp_port: number;
  use_tls: boolean;
  username: string;
  password: string;
  imap_host?: string;
  imap_port?: number;
  imap_use_tls?: boolean;
  imap_sync_enabled: boolean;
}

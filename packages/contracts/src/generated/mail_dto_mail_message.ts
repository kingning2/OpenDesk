export interface MailDtoMailMessage {
  id: string;
  customer_id?: string;
  template_id?: string;
  account_id?: string;
  status: string;
  direction: string;
  subject: string;
  body_text: string;
  body_html?: string;
  to_address?: string;
  from_address?: string;
  error_message?: string;
  sent_at?: string;
  received_at?: string;
  imap_uid?: number;
  imap_folder?: string;
  rfc_message_id?: string;
  in_reply_to?: string;
  references?: string;
  is_favorite?: boolean;
  open_tracking_id?: string;
  created_at: string;
  updated_at: string;
}

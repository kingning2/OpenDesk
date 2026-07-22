export interface MailIpcSendRequest {
  customer_id?: string;
  to_address: string;
  account_id: string;
  template_id?: string;
  subject: string;
  body_text: string;
  body_html?: string;
  open_tracking_enabled?: boolean;
}

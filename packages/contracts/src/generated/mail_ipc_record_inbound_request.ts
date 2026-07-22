export interface MailIpcRecordInboundRequest {
  customer_id: string;
  from_address: string;
  from_name?: string;
  subject: string;
  body_text: string;
  body_html?: string;
  received_at: string;
  rfc_message_id?: string;
  in_reply_to?: string;
}

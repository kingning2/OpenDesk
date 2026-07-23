export interface MailEventInboundReceived {
  event_id: string;
  occurred_at: string;
  message_id: string;
  account_id: string;
  customer_id?: string;
  direction: string;
  subject?: string;
  from_address?: string;
}

export interface MailIpcMessageListRequest {
  direction: string;
  account_id?: string;
  customer_id?: string;
  query?: string;
  limit?: number;
  offset?: number;
}

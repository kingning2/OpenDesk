export interface CustomerIpcListResponse {
  ok: boolean;
  customers_json: string;
  total: number;
  trace_id?: string;
}

export interface ChannelSidecarQrStartResponse {
  ok: boolean;
  status: string;
  session_id?: string;
  qr_base64?: string;
  detail?: string;
  trace_id?: string;
}

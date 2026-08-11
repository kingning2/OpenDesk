export interface ChannelIpcQrStartResponse {
  ok: boolean;
  status: string;
  session_id?: string;
  qr_base64?: string;
  detail?: string;
}

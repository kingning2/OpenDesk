export interface RuntimeSidecarLlmTestConnectionRequest {
  provider: string;
  base_url?: string;
  model_id: string;
  api_key: string;
  trace_id?: string;
}

export interface RuntimeIpcLlmSettingsSaveResponse {
  provider: string;
  base_url?: string;
  model_id: string;
  configured: boolean;
  has_api_key: boolean;
}

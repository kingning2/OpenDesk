export interface RuntimeIpcLlmSettingsSaveRequest {
  provider: string;
  base_url?: string;
  model_id: string;
  api_key: string;
  tools_enabled: boolean;
  memory_enabled: boolean;
  knowledge_enabled: boolean;
}

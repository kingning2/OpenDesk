export interface AiProvider {
  id: string;
  kind: string;
  name: string;
  base_url?: string;
  default_model?: string;
}

import type { LlmProvider } from "./llm_provider";

export interface LlmIpcClassifyRequest {
  text: string;
  scenario?: string;
  options: string[];
  provider?: LlmProvider;
  trace_id?: string;
}

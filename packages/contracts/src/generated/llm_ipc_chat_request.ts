import type { LlmMessage } from "./llm_message";
import type { LlmProvider } from "./llm_provider";

export interface LlmIpcChatRequest {
  messages: LlmMessage[];
  provider: LlmProvider;
  trace_id?: string;
}

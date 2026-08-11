import type { AiAccount } from "./ai_account";
import type { AiProvider } from "./ai_provider";

export interface AiIpcConfigResponse {
  providers: AiProvider[];
  accounts: AiAccount[];
}

import type { AiAccount } from "./ai_account";
import type { AiProvider } from "./ai_provider";

export interface AiIpcConfigRequest {
  providers: AiProvider[];
  accounts: AiAccount[];
}

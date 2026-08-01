import { useState } from "react";
import { agentPing } from "@desk/platform/ipc/agent";

export function useAgentPing() {
  const [status, setStatus] = useState("Ready.");
  const [loading, setLoading] = useState(false);

  async function ping() {
    setLoading(true);
    try {
      const result = await agentPing({ trace_id: crypto.randomUUID() });
      setStatus(result.ok ? `LLM ok (${result.trace_id ?? "no trace"})` : "LLM connection failed");
    } catch (error) {
      const message = error instanceof Error ? error.message : "LLM unavailable";
      setStatus(message);
    } finally {
      setLoading(false);
    }
  }

  return { status, loading, ping };
}

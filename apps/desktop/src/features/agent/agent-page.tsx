import { Card, CardContent, CardHeader, CardTitle } from "@desk/ui";
import { useAgentPing } from "./use-agent-ping";

export function AgentPage() {
  const { status, loading, ping } = useAgentPing();

  return (
    <Card variant="glass" className="w-full max-w-lg">
      <CardHeader>
        <CardTitle>Agent</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-[length:var(--text-sm)] text-muted-foreground">{status}</p>
        <button
          type="button"
          disabled={loading}
          onClick={ping}
          className="rounded-[var(--radius-md)] bg-primary px-4 py-2 text-[length:var(--text-sm)] text-primary-foreground disabled:opacity-60"
        >
          {loading ? "Pinging..." : "Ping sidecar"}
        </button>
      </CardContent>
    </Card>
  );
}

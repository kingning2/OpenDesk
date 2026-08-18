/**
 * Agent Feature 页。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { Button, Card, CardContent, CardHeader, CardTitle } from "@desk/ui";

import { useAgentPing } from "./use-agent-ping";

/**
 * Agent 连通性演示页。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 页面节点
 */
export function AgentPage() {
  const { status, loading, ping } = useAgentPing();

  return (
    <>
      <p className="text-(length:--text-sm) text-muted-foreground">Sidecar 连通性垂直切片</p>
      <Card variant="glass" className="w-full">
        <CardHeader>
          <CardTitle>Agent</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-[length:var(--text-sm)] text-muted-foreground">{status}</p>
          <Button disabled={loading} onClick={ping}>
            {loading ? "请求中…" : "Ping sidecar"}
          </Button>
        </CardContent>
      </Card>
    </>
  );
}

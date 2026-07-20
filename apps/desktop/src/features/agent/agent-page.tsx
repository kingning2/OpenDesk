/**
 * Agent Feature 页。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */

import { Card, CardContent, CardHeader, CardTitle, PageScaffold } from "@desk/ui";

import { useT } from "../../i18n";
import { useAgentPing } from "./use-agent-ping";

/**
 * Agent 连通性演示页。
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @returns 页面节点
 */
export function AgentPage() {
  const t = useT();
  const { status, loading, ping } = useAgentPing();

  return (
    <PageScaffold subtitle={t("agent.subtitle")}>
      <Card variant="glass" className="w-full">
        <CardHeader>
          <CardTitle>{t("agent.title")}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-[length:var(--text-sm)] text-muted-foreground">{status}</p>
          <button
            type="button"
            disabled={loading}
            onClick={ping}
            className="rounded-[var(--radius-md)] bg-primary px-4 py-2 text-[length:var(--text-sm)] text-primary-foreground disabled:opacity-60"
          >
            {loading ? t("agent.pinging") : t("agent.ping")}
          </button>
        </CardContent>
      </Card>
    </PageScaffold>
  );
}

/**
 * 首页。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { Card, CardContent, CardHeader, CardTitle, PageScaffold } from "@desk/ui";

/**
 * 应用首页。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 首页节点
 */
export function HomePage() {
  return (
    <PageScaffold>
      <Card variant="glass" className="w-full">
        <CardHeader>
          <CardTitle>OpenDesk</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-[length:var(--text-sm)] text-muted-foreground">
            架构脚手架 — 请从侧栏选择功能。
          </p>
        </CardContent>
      </Card>
    </PageScaffold>
  );
}

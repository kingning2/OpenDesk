/**
 * 首页。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */

import { Card, CardContent, CardHeader, CardTitle, PageScaffold } from "@desk/ui";

import { useT } from "../../i18n";

/**
 * 应用首页。
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @returns 首页节点
 */
export function HomePage() {
  const t = useT();

  return (
    <PageScaffold>
      <Card variant="glass" className="w-full">
        <CardHeader>
          <CardTitle>{t("home.title")}</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-[length:var(--text-sm)] text-muted-foreground">
            {t("home.description")}
          </p>
        </CardContent>
      </Card>
    </PageScaffold>
  );
}

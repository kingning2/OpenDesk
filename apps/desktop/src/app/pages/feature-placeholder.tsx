/**
 * Feature 占位页。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */

import { Card, CardContent, CardDescription, CardHeader, CardTitle, PageScaffold } from "@desk/ui";

import { useT } from "../../i18n";

/**
 * 占位页属性。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */
export interface FeaturePlaceholderPageProps {
  /** 标题 i18n key。 */
  titleKey: string;
  /** 描述 i18n key。 */
  descriptionKey?: string;
}

/**
 * 开发中 Feature 的占位展示。
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @param props - 见 {@link FeaturePlaceholderPageProps}
 * @returns 占位页节点
 */
export function FeaturePlaceholderPage({ titleKey, descriptionKey }: FeaturePlaceholderPageProps) {
  const t = useT();

  return (
    <PageScaffold>
      <Card variant="glass" className="w-full">
        <CardHeader>
          <CardTitle>{t(titleKey)}</CardTitle>
          {descriptionKey ? <CardDescription>{t(descriptionKey)}</CardDescription> : null}
        </CardHeader>
        <CardContent>
          <p className="text-[length:var(--text-sm)] text-muted-foreground">
            {t("placeholder.developing")}
          </p>
        </CardContent>
      </Card>
    </PageScaffold>
  );
}

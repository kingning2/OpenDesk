import { Card, CardContent, CardDescription, CardHeader, CardTitle, PageScaffold } from "@desk/ui";

export interface FeaturePlaceholderPageProps {
  featureName: string;
  description?: string;
}

export function FeaturePlaceholderPage({ featureName, description }: FeaturePlaceholderPageProps) {
  return (
    <PageScaffold>
      <Card variant="glass" className="w-full">
        <CardHeader>
          <CardTitle>{featureName}</CardTitle>
          {description ? <CardDescription>{description}</CardDescription> : null}
        </CardHeader>
        <CardContent>
          <p className="text-[length:var(--text-sm)] text-muted-foreground">This feature is under development.</p>
        </CardContent>
      </Card>
    </PageScaffold>
  );
}

import { Card, CardContent, CardHeader, CardTitle, PageScaffold } from "@desk/ui";

export function HomePage() {
  return (
    <PageScaffold>
      <Card variant="glass" className="w-full">
        <CardHeader>
          <CardTitle>OpenDesk</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-[length:var(--text-sm)] text-muted-foreground">
            Architecture scaffold — select a feature from the sidebar.
          </p>
        </CardContent>
      </Card>
    </PageScaffold>
  );
}

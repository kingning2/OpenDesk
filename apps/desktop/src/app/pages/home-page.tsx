import { Card, CardContent, CardHeader, CardTitle } from "@desk/ui";

export function HomePage() {
  return (
    <Card variant="glass" className="w-full max-w-lg">
      <CardHeader>
        <CardTitle>OpenDesk</CardTitle>
      </CardHeader>
      <CardContent>
        <p className="text-[length:var(--text-sm)] text-muted-foreground">
          Architecture scaffold — select a feature from the sidebar.
        </p>
      </CardContent>
    </Card>
  );
}

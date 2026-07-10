import { Card, CardContent, CardHeader, CardTitle, ThemeProvider } from "@desk/ui";
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./globals.css";

export function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <ThemeProvider>
      <main className="flex min-h-screen items-center justify-center p-6">
        <Card variant="glass" className="w-full max-w-md">
          <CardHeader>
            <CardTitle>OpenDesk</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <p className="text-[length:var(--text-sm)] text-muted-foreground">
              Architecture scaffold is ready.
            </p>
            <form
              className="flex gap-2"
              onSubmit={(e) => {
                e.preventDefault();
                greet();
              }}
            >
              <input
                id="greet-input"
                className="flex-1 rounded-[var(--radius-md)] border border-border bg-background px-3 py-2 text-[length:var(--text-sm)]"
                onChange={(e) => setName(e.currentTarget.value)}
                placeholder="Enter a name..."
              />
              <button
                type="submit"
                className="rounded-[var(--radius-md)] bg-primary px-4 py-2 text-[length:var(--text-sm)] text-primary-foreground"
              >
                Greet
              </button>
            </form>
            {greetMsg ? <p className="text-[length:var(--text-sm)]">{greetMsg}</p> : null}
          </CardContent>
        </Card>
      </main>
    </ThemeProvider>
  );
}

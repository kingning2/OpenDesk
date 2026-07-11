import { NavLink, Outlet } from "react-router";
import { ThemeProvider } from "@desk/ui";
import { agentFeature } from "@feature/agent";

const navItems = [
  { to: "/", label: "Home", end: true },
  { to: agentFeature.path, label: "Agent" },
];

export function AppShell() {
  return (
    <ThemeProvider>
      <div className="flex min-h-screen">
        <aside className="w-52 border-r border-border p-4">
          <p className="mb-4 text-[length:var(--text-sm)] font-medium">OpenDesk</p>
          <nav className="flex flex-col gap-2">
            {navItems.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                end={item.end}
                className={({ isActive }) =>
                  [
                    "rounded-[var(--radius-md)] px-3 py-2 text-[length:var(--text-sm)]",
                    isActive ? "bg-primary text-primary-foreground" : "text-muted-foreground",
                  ].join(" ")
                }
              >
                {item.label}
              </NavLink>
            ))}
          </nav>
        </aside>
        <main className="flex flex-1 items-start justify-center p-6">
          <Outlet />
        </main>
      </div>
    </ThemeProvider>
  );
}

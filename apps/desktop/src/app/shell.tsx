import { NavLink, useNavigate } from "react-router";
import {
  AppLayout,
  IconButton,
  MainPanel,
  NavRail,
  NavRailNav,
  navRailItemVariants,
  TabBar,
  ThemeProvider,
  ThemeToggle,
  TitleBar,
} from "@desk/ui";
import { Settings } from "@desk/ui/icons";
import { closeWindow, getPlatform, minimizeWindow, startWindowDrag, toggleMaximizeWindow } from "@desk/platform";
import { navItems } from "../route/nav-registry";
import { useWorkspaceTabs } from "./use-workspace-tabs";
import { WorkspaceOutlet } from "./workspace-outlet";

export function AppShell() {
  const platform = getPlatform();
  const navigate = useNavigate();
  const { tabs, activePath, openPaths, ensureTab, selectTab, closeTab, addTab } = useWorkspaceTabs();

  return (
    <ThemeProvider defaultTheme="dark">
      <div className="flex h-screen w-full flex-col overflow-hidden bg-shell">
        <TitleBar
          platform={platform}
          actions={
            <>
              <IconButton
                label="Settings"
                title="设置"
                onClick={() => {
                  ensureTab("/settings");
                  navigate("/settings");
                }}
              >
                <Settings className="size-3.5" />
              </IconButton>
              <ThemeToggle size="compact" />
            </>
          }
          tabs={
            <TabBar
              embedded
              items={tabs}
              activePath={activePath}
              onSelect={selectTab}
              onClose={closeTab}
              onAdd={addTab}
            />
          }
          onStartDrag={() => void startWindowDrag()}
          onMinimize={() => void minimizeWindow()}
          onToggleMaximize={() => void toggleMaximizeWindow()}
          onClose={() => void closeWindow()}
        />
        <AppLayout
          sidebar={
            <NavRail>
              <NavRailNav>
                {navItems.map((item) => {
                  const Icon = item.icon;
                  return (
                    <NavLink
                      key={item.id}
                      to={item.path}
                      end={item.end}
                      title={item.label}
                      onClick={() => ensureTab(item.path)}
                      className={({ isActive }) => navRailItemVariants({ active: isActive })}
                    >
                      {Icon ? <Icon className="size-[1.125rem] shrink-0" aria-hidden /> : null}
                      <span className="max-w-full truncate">{item.label}</span>
                    </NavLink>
                  );
                })}
              </NavRailNav>
            </NavRail>
          }
        >
          <MainPanel>
            <WorkspaceOutlet openPaths={openPaths} activePath={activePath} />
          </MainPanel>
        </AppLayout>
      </div>
    </ThemeProvider>
  );
}

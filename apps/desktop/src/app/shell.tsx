import { NavLink, Outlet, useLocation } from "react-router";
import {
  AnimatePresence,
  AppLayout,
  FadeSlide,
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
import { LicensePlanBadge } from "@feature/license";
import { navItems } from "../route/nav-registry";
import { useWorkspaceTabs } from "./use-workspace-tabs";

export function AppShell() {
  const platform = getPlatform();
  const location = useLocation();
  const { tabs, activePath, ensureTab, selectTab, closeTab, addTab } = useWorkspaceTabs();

  return (
    <ThemeProvider defaultTheme="dark">
      <div className="flex h-screen w-full flex-col overflow-hidden bg-shell">
        <TitleBar
          platform={platform}
          actions={
            <>
              <IconButton label="Settings" title="Settings">
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
            <NavRail className="h-full min-h-0">
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
              <LicensePlanBadge />
            </NavRail>
          }
        >
          <MainPanel>
            <AnimatePresence mode="wait">
              <FadeSlide key={location.pathname} className="flex min-h-0 flex-1 flex-col overflow-hidden">
                <Outlet />
              </FadeSlide>
            </AnimatePresence>
          </MainPanel>
        </AppLayout>
      </div>
    </ThemeProvider>
  );
}

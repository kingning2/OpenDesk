/**
 * 桌面应用主壳：窗口标题栏 + 侧栏导航 + 工作区。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */

import { NavLink, useNavigate } from "react-router";
import { IconButton, ThemeProvider, ThemeToggle } from "@desk/ui";
import { Settings } from "@desk/ui/icons";
import { closeWindow, getPlatform, minimizeWindow, startWindowDrag, toggleMaximizeWindow } from "@desk/platform";
import { LicensePlanBadge } from "@feature/license";
import { useI18n } from "../i18n";
import { navItems } from "../route/nav-registry";
import {
  AppLayout,
  MainPanel,
  NavRail,
  NavRailNav,
  navRailItemVariants,
  TabBar,
} from "./layout";
import { TitleBar } from "./title-bar";
import { useWorkspaceTabs } from "./use-workspace-tabs";
import { WorkspaceOutlet } from "./workspace-outlet";

/**
 * 桌面应用主壳。
 *
 * 负责：窗口 TitleBar、侧栏导航、工作区标签与内容出口。
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @returns 应用壳节点
 */
export function AppShell() {
  const platform = getPlatform();
  const navigate = useNavigate();
  const { t } = useI18n();
  const { tabs, activePath, openPaths, ensureTab, selectTab, closeTab, addTab } = useWorkspaceTabs();

  return (
    <ThemeProvider defaultTheme="dark">
      <div className="flex h-screen w-full flex-col overflow-hidden bg-shell">
        <TitleBar
          platform={platform}
          actions={
            <>
              <IconButton
                label={t("shell.settings")}
                title={t("shell.settings")}
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
            <NavRail className="h-full min-h-0">
              <NavRailNav>
                {navItems.map((item) => {
                  const Icon = item.icon;
                  const label = t(item.labelKey);
                  return (
                    <NavLink
                      key={item.id}
                      to={item.path}
                      end={item.end}
                      title={label}
                      onClick={() => ensureTab(item.path)}
                      className={({ isActive }) => navRailItemVariants({ active: isActive })}
                    >
                      {Icon ? <Icon className="size-[1.125rem] shrink-0" aria-hidden /> : null}
                      <span className="max-w-full truncate">{label}</span>
                    </NavLink>
                  );
                })}
              </NavRailNav>
              <LicensePlanBadge />
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

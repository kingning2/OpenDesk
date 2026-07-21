/**
 * 桌面应用主壳：窗口标题栏 + 侧栏导航 + 工作区。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { useEffect, useState } from "react";
import { NavLink } from "react-router";
import { IconButton, ThemeProvider, ThemeToggle } from "@desk/ui";
import { Settings } from "@desk/ui/icons";
import {
  closeWindow,
  getPlatform,
  minimizeWindow,
  startWindowDrag,
  subscribeWindowMaximized,
  toggleMaximizeWindow,
} from "@desk/platform";
import { LicensePlanBadge } from "@feature/license";
import { SettingsDialogProvider, useSettingsDialog } from "@feature/setting";
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
 * 壳内层：依赖 {@link SettingsDialogProvider}。
 *
 * @author coisini
 * @created 2026-07-21
 *
 * @returns 壳内容节点
 */
function AppShellInner() {
  const platform = getPlatform();
  const [isMaximized, setIsMaximized] = useState(false);
  const { t } = useI18n();
  const { openSettings } = useSettingsDialog();
  const { tabs, activePath, openPaths, ensureTab, selectTab, closeTab, addTab } = useWorkspaceTabs();

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    void subscribeWindowMaximized((maximized) => {
      if (!cancelled) {
        setIsMaximized(maximized);
      }
    }).then((unsubscribe) => {
      if (cancelled) {
        unsubscribe();
        return;
      }
      unlisten = unsubscribe;
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return (
    <div className="flex h-screen w-full flex-col overflow-hidden bg-shell">
      <TitleBar
        platform={platform}
        isMaximized={isMaximized}
        actions={
          <>
            <IconButton
              label={t("shell.settings")}
              title={t("shell.settings")}
              onClick={openSettings}
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
  );
}

/**
 * 桌面应用主壳。
 *
 * 负责：窗口 TitleBar、侧栏导航、工作区标签与内容出口、设置弹窗。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 应用壳节点
 */
export function AppShell() {
  return (
    <ThemeProvider defaultTheme="dark">
      <SettingsDialogProvider>
        <AppShellInner />
      </SettingsDialogProvider>
    </ThemeProvider>
  );
}

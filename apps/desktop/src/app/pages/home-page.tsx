/**
 * 首页 — 欢迎与工作区引导。
 *
 * 具体功能入口已移至左侧 Aceternity 分组侧栏，首页不再重复罗列路由。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { useWorkspaceNav } from "../use-workspace-tabs";
import { Button, Card, CardContent, PageScaffold } from "@desk/ui";
import { CHANNEL_MANAGE_ROOT } from "@desk/platform/compile";
import { LayoutDashboard } from "@desk/ui/icons";

/**
 * 应用首页。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 首页节点
 */
export function HomePage() {
  const { selectTab } = useWorkspaceNav();

  return (
    <PageScaffold
      title="欢迎使用 DingDa"
      subtitle="从左侧导航进入各功能模块；常用入口可从仪表盘开始"
      ambient="spotlight"
      extra={
        <Button variant="outline" onClick={() => selectTab(CHANNEL_MANAGE_ROOT)}>
          <LayoutDashboard className="size-4" aria-hidden />
          进入首页
        </Button>
      }
    >
      <Card variant="glass" className="max-w-xl">
        <CardContent className="space-y-2 p-6 text-[length:var(--text-sm)] text-muted-foreground">
          <p>工作区左侧为分组菜单：点击分组标题可展开/收起子项，状态会自动记住。</p>
          <p>点击左侧菜单即可切换页面。</p>
        </CardContent>
      </Card>
    </PageScaffold>
  );
}

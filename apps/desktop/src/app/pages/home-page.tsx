/**
 * 首页 — 业务入口聚合页。
 *
 * 管理子页入口由 `@platform-routes` 按编译平台注入（闲鱼构建含完整入口）。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { useNavigate } from "react-router";
import { Button, Card, CardContent } from "@desk/ui";
import { managePath } from "@desk/platform/compile";
import { homeManageNav } from "@platform-routes";

/**
 * 应用首页。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 首页节点
 */
export function HomePage() {
  const navigate = useNavigate();

  return (
    <>
      <p className="text-(length:--text-sm) text-muted-foreground">业务入口 — 选择要管理的功能</p>
      <div className="grid grid-cols-2 gap-3 md:grid-cols-3 xl:grid-cols-4">
        {homeManageNav.map((item) => {
          const Icon = item.icon;
          return (
            <Button
              key={item.key}
              variant="ghost"
              onClick={() => navigate(managePath(item.key))}
              className="group h-auto p-0 text-left"
            >
              <Card variant="glass" className="h-full w-full transition-colors group-hover:border-primary/40">
                <CardContent className="flex flex-col gap-2.5 p-4">
                  <div className="flex size-9 items-center justify-center rounded-[var(--radius-md)] bg-primary/10 text-primary">
                    <Icon className="size-4" aria-hidden />
                  </div>
                  <div>
                    <p className="text-[length:var(--text-sm)] font-medium">{item.label}</p>
                    <p className="mt-0.5 line-clamp-2 text-[length:var(--text-xs)] leading-relaxed text-muted-foreground">
                      {item.description}
                    </p>
                  </div>
                </CardContent>
              </Card>
            </Button>
          );
        })}
      </div>
    </>
  );
}
